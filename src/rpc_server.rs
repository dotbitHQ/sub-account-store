use super::{
    default_store::DefaultStoreMultiTree,
    structures::{
        DefaultStoreMultiSMT, MemoryStoreSMT, Opt, Pair, Response, ResponseSequence, SmtKey,
        SmtProof, SmtRoot, SmtValue,
    },
    utils::slice_to_hex_string,
};
use crate::utils::get_empty_compiled_proof;
use jsonrpsee::{
    core::{async_trait, Error},
    proc_macros::rpc,
};
use log::{error, info, trace, warn};
use rayon::prelude::*;
use rocksdb::{prelude::Iterate, OptimisticTransactionDB};
use rocksdb::{Direction, IteratorMode};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use sparse_merkle_tree::{traits::Value, H256};
use std::collections::HashMap;

pub struct RpcServerImpl {
    db: OptimisticTransactionDB,
}

impl RpcServerImpl {
    pub fn new(db: OptimisticTransactionDB) -> Self {
        Self { db }
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OperationResult(bool);

#[rpc(server)]
pub trait Rpc {
    #[method(name = "update_memory_smt")]
    async fn build_memory_smt(
        &self,
        opt: Opt,
        smt_name: &str,
        data: Vec<Pair>,
    ) -> Result<Response, Error>;

    #[method(name = "update_db_smt")]
    async fn update_rocksdb_smt(
        &self,
        opt: Opt,
        smt_name: &str,
        data: Vec<Pair>,
    ) -> Result<Response, Error>;

    #[method(name = "update_db_smt_middle")]
    async fn update_rocksdb_smt_sequence(
        &self,
        opt: Opt,
        smt_name: &str,
        data: Vec<Pair>,
    ) -> Result<ResponseSequence, Error>;

    #[method(name = "get_smt_root")]
    async fn get_smt_root(&self, smt_name: &str) -> Result<SmtRoot, Error>;

    #[method(name = "delete_smt")]
    async fn delete_smt(&self, smt_name: &str) -> Result<(), Error>;
}

#[async_trait]
impl RpcServer for RpcServerImpl {
    async fn build_memory_smt(
        &self,
        opt: Opt,
        _smt_name: &str,
        kvs_in: Vec<Pair>,
    ) -> Result<Response, Error> {
        info!("building smt in memory");
        let get_root = opt.get_root;
        let get_proof = opt.get_proof;
        trace!("bms: get_root={}, get_proof={}", &get_root, &get_proof);

        //check if data is empty
        if kvs_in.is_empty() {
            warn!("the keys in the request are empty");
            return Ok(Response::default());
        }

        let kvs: Vec<(H256, SmtValue)> = kvs_in
            .clone()
            .into_iter()
            .map(|k| (k.key.0.into(), k.value))
            .collect();

        let mut memory_store_smt = match MemoryStoreSMT::new_with_store(Default::default()) {
            Ok(m) => m,
            Err(e) => {
                error!("cannot initialize memory store : {}", &e);
                return Err(Error::Custom(e.to_string()));
            }
        };

        //bulid the tree and get the root
        let smt_root = match memory_store_smt.update_all(kvs) {
            Ok(root) => {
                info!("building success");
                root.into()
            }
            Err(e) => {
                error!("building smt in memory failed! : {}", &e);
                return Err(Error::Custom(e.to_string()));
            }
        };
        // let smt_root = memory_store_smt
        //     .update_all(kvs)
        //     .expect("cannot create tree")
        //     .into();

        let smt_proofs = if !get_proof {
            let mut smt_proofs = Vec::new();
            let k = slice_to_hex_string(SmtKey::default().0.as_slice());
            let v = slice_to_hex_string(SmtProof::default().0.as_slice());
            smt_proofs.push((k, v));
            smt_proofs
        } else {
            let keys: Vec<H256> = kvs_in.clone().into_iter().map(|k| k.key.0.into()).collect();

            keys.par_iter()
                .filter_map(|k| {
                    let mut vec = Vec::new();
                    vec.push(*k);
                    let proof = match memory_store_smt.merkle_proof(vec.clone()) {
                        Ok(proof) => Some(proof),
                        Err(e) => {
                            error!(
                                "get merkle proof failed!  key= {} : err = {}",
                                slice_to_hex_string(k.as_slice()),
                                &e
                            );
                            None
                        }
                    };
                    if let Some(p) = proof {
                        match p.clone().compile(vec) {
                            Ok(compiled_proof) => {
                                let k = slice_to_hex_string(k.to_h256().as_slice());
                                let v = slice_to_hex_string(compiled_proof.0.as_slice());
                                Some((k, v))
                            }
                            Err(e) => {
                                error!("unable to generate compiled proof : {}", &e);
                                None
                            }
                        }
                    } else {
                        None
                    }
                })
                .collect()
        };

        let hashmap_proofs: HashMap<_, _> = smt_proofs.into_par_iter().collect();

        let r = if !get_root {
            Response {
                root: SmtRoot::default(),
                proofs: hashmap_proofs,
            }
        } else {
            Response {
                root: smt_root,
                proofs: hashmap_proofs,
            }
        };

        Ok(r)
    }
    async fn update_rocksdb_smt(
        &self,
        opt: Opt,
        smt_name: &str,
        kvs_in: Vec<Pair>,
    ) -> Result<Response, Error> {
        info!("update smt in the database in order");
        let get_root = opt.get_root;
        let get_proof = opt.get_proof;

        let tx = self.db.transaction_default();
        let mut rocksdb_store_smt = match DefaultStoreMultiSMT::new_with_store(
            DefaultStoreMultiTree::new(smt_name.as_bytes(), &tx),
        ) {
            Ok(r) => r,
            Err(e) => {
                error!("cannot initialize database store : {}", &e);
                return Err(Error::Custom(e.to_string()));
            }
        };

        let kvs: Vec<(H256, SmtValue)> = kvs_in
            .clone()
            .into_par_iter()
            .map(|k| (k.key.0.into(), k.value))
            .collect();

        let smt_root = match rocksdb_store_smt.update_all(kvs.into()) {
            Ok(root) => {
                info!("update success");
                root.into()
            }
            Err(e) => {
                error!("building smt in database failed! : {}", &e);
                return Err(Error::Custom(e.to_string()));
            }
        };

        tx.commit().expect("db commit error");

        let smt_proofs = if !get_proof {
            let mut smt_proofs = vec![];
            let k = format!("{:?}", SmtKey::default());
            let v = format!("{:?}", SmtProof::default());

            //smt_proofs.push((, SmtProof::default()));
            smt_proofs.push((k, v));
            smt_proofs
        } else {
            let keys: Vec<H256> = kvs_in
                .clone()
                .into_par_iter()
                .map(|k| k.key.0.into())
                .collect();

            keys.par_iter()
                .filter_map(|k| {
                    let mut vec = vec![];
                    vec.push(*k);
                    let proof = match rocksdb_store_smt.merkle_proof(vec.clone()) {
                        Ok(proof) => Some(proof),
                        Err(e) => {
                            error!(
                                "get merkle proof failed!  key= {} : err = {}",
                                slice_to_hex_string(k.as_slice()),
                                &e
                            );
                            None
                        }
                    };

                    if let Some(p) = proof {
                        match p.clone().compile(vec) {
                            Ok(compiled_proof) => {
                                let k = slice_to_hex_string(k.to_h256().as_slice());
                                let v = slice_to_hex_string(compiled_proof.0.as_slice());
                                Some((k, v))
                            }
                            Err(e) => {
                                error!("unable to generate compiled proof : {}", &e);
                                None
                            }
                        }
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        };

        let hashmap_proofs: HashMap<_, _> = smt_proofs.into_par_iter().collect();

        let r = if !get_root {
            Response {
                root: SmtRoot::default(),
                proofs: hashmap_proofs,
            }
        } else {
            Response {
                root: smt_root,
                proofs: hashmap_proofs,
            }
        };
        Ok(r)
    }

    async fn update_rocksdb_smt_sequence(
        &self,
        opt: Opt,
        smt_name: &str,
        kvs_in: Vec<Pair>,
    ) -> Result<ResponseSequence, Error> {
        info!("update smt in the database in order");

        let get_root = opt.get_root;
        let get_proof = opt.get_proof;

        let tx = self.db.transaction_default();
        let mut rocksdb_store_smt = match DefaultStoreMultiSMT::new_with_store(
            DefaultStoreMultiTree::new(smt_name.as_bytes(), &tx),
        ) {
            Ok(r) => r,
            Err(e) => {
                error!("cannot initialize database store : {}", &e);
                return Err(Error::Custom(e.to_string()));
            }
        };
        let kvs: Vec<(H256, SmtValue)> = kvs_in
            .clone()
            .into_par_iter()
            .map(|k| (k.key.0.into(), k.value))
            .collect();

        let mut hashmap_roots = HashMap::new();
        let mut hashmap_proofs = HashMap::new();
        for (k, v) in kvs {
            {
                match rocksdb_store_smt.update(k, v.clone()) {
                    Ok(_) => {}
                    Err(e) => {
                        error!(
                            "cannot update smt, err = {}, key = {}, value = {}, ",
                            &e,
                            slice_to_hex_string(&k.as_slice()),
                            slice_to_hex_string(&v.as_ref())
                        );
                        continue;
                    }
                };
                tx.commit().expect("db commit error");
            };
            let smt_root = rocksdb_store_smt.root();

            let compiled_proof = if !get_proof {
                get_empty_compiled_proof()
            } else {
                let mut vec = vec![];
                vec.push(k);
                let proof = match rocksdb_store_smt.merkle_proof(vec.clone()) {
                    Ok(proof) => Some(proof),
                    Err(e) => {
                        error!(
                            "get merkle proof failed!  key= {} : err = {}",
                            slice_to_hex_string(k.as_slice()),
                            &e
                        );
                        None
                    }
                };
                if let Some(p) = proof {
                    match p.clone().compile(vec) {
                        Ok(compiled_proof) => compiled_proof,
                        Err(e) => {
                            error!("unable to generate compiled proof : {}", &e);
                            get_empty_compiled_proof()
                        }
                    }
                } else {
                    get_empty_compiled_proof()
                }
            };

            let root = slice_to_hex_string(smt_root.as_slice());
            let proof = slice_to_hex_string(compiled_proof.0.as_slice());
            hashmap_roots.insert(slice_to_hex_string(k.as_slice()), root);
            hashmap_proofs.insert(slice_to_hex_string(k.as_slice()), proof);
        }

        let r = if !get_root {
            ResponseSequence {
                roots: hashmap_roots,
                proofs: hashmap_proofs,
            }
        } else {
            ResponseSequence {
                roots: hashmap_roots,
                proofs: hashmap_proofs,
            }
        };
        Ok(r)
    }
    async fn get_smt_root(&self, smt_name: &str) -> Result<SmtRoot, Error> {
        info!("get smt root");
        let snapshot = self.db.snapshot();
        let rocksdb_store_smt =
            match DefaultStoreMultiSMT::new_with_store(DefaultStoreMultiTree::<_, ()>::new(
                smt_name.as_bytes(),
                &snapshot,
            )) {
                Ok(r) => r,
                Err(e) => {
                    error!("cannot initialize database store : {}", &e);
                    return Err(Error::Custom(e.to_string()));
                }
            };

        let smt_root = rocksdb_store_smt.root().into();
        Ok(smt_root)
    }

    async fn delete_smt(&self, smt_name: &str) -> Result<(), Error> {
        info!("delete smt tree : {}", &smt_name);
        // OptimisticTransactionDB does not support delete_range, so we have to iterate all keys and update them to zero as a workaround
        let snapshot = self.db.snapshot();
        let prefix = smt_name.as_bytes();
        let prefix_len = prefix.len();
        let leaf_key_len = prefix_len + 32;
        let kvs: Vec<(H256, SmtValue)> = snapshot
            .iterator(IteratorMode::From(prefix, Direction::Forward))
            .take_while(|(k, _)| k.starts_with(prefix))
            .filter_map(|(k, _)| {
                if k.len() != leaf_key_len {
                    None
                } else {
                    match k[prefix_len..].try_into() {
                        Ok(r) => {
                            let leaf_key: [u8; 32] = r;
                            Some((leaf_key.into(), SmtValue::zero()))
                        }
                        Err(e) => {
                            warn!("cannot try into: {}", &e);
                            None
                        }
                    }
                }
            })
            .collect();

        let tx = self.db.transaction_default();
        let mut rocksdb_store_smt = match DefaultStoreMultiSMT::new_with_store(
            DefaultStoreMultiTree::new(smt_name.as_bytes(), &tx),
        ) {
            Ok(r) => r,
            Err(e) => {
                error!("cannot initialize database store : {}", &e);
                return Err(Error::Custom(e.to_string()));
            }
        };
        let smt_root = match rocksdb_store_smt.update_all(kvs) {
            Ok(root) => {
                info!("update success");
                root
            }
            Err(e) => {
                error!("building smt in database failed! : {}", &e);
                return Err(Error::Custom(e.to_string()));
            }
        };

        tx.commit().expect("db commit error");
        if smt_root.eq(&H256::zero()) {
            info!("delete smt tree {}: success", &smt_name);
        } else {
            error!("delete smt tree {}: fail", &smt_name);
        }
        Ok(())
    }
}
