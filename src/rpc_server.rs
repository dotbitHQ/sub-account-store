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

use log::{debug, error, info, warn};
use rayon::prelude::*;
use rocksdb::{prelude::Iterate, OptimisticTransaction, OptimisticTransactionDB};
use rocksdb::{Direction, IteratorMode};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use sparse_merkle_tree::{traits::Value, H256};
use std::collections::HashMap;

const CHUNK_SIZE: usize = 5000;
const MAX_DISPLAY_NUMS: usize = 5;

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
    async fn delete_smt(&self, smt_name: &str) -> Result<bool, Error>;
}

#[async_trait]
impl RpcServer for RpcServerImpl {
    async fn build_memory_smt(
        &self,
        opt: Opt,
        _smt_name: &str,
        kvs_in: Vec<Pair>,
    ) -> Result<Response, Error> {
        let (get_root, get_proof) = (opt.get_root, opt.get_proof);

        info!(
            "building smt in memory start: get_root = {}, get_proof = {}, keys_len = {}, {}",
            get_root,
            get_proof,
            kvs_in.len(),
            generate_kvs_info(&kvs_in)
        );

        debug!("{}", generate_kvs_debug(&kvs_in));

        //check if data is empty
        if kvs_in.is_empty() {
            warn!("empty key-value pairs in the request");
            return Ok(Response::default());
        }

        let kvs: Vec<(H256, SmtValue)> = kvs_in
            .clone()
            .into_iter()
            .map(|k| (k.key.0.into(), k.value))
            .collect();

        info!("get the storage handle");
        let mut memory_store_smt = match MemoryStoreSMT::new_with_store(Default::default()) {
            Ok(m) => m,
            Err(e) => {
                error!("cannot initialize memory store : {}", &e);
                return Err(Error::Custom(e.to_string()));
            }
        };

        //build the tree and get the root
        info!("update key-value pairs to the memory");
        let smt_root = match memory_store_smt.update_all(kvs) {
            Ok(root) => {
                info!("update successful");
                root.into()
            }
            Err(e) => {
                error!("update failed! : {}", &e);
                return Err(Error::Custom(e.to_string()));
            }
        };

        info!("generate proof");
        let smt_proofs = if !get_proof {
            default_merkel_proof()
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
        debug!("response = {}", generate_response_debug(&r));
        info!("building smt in memory end");

        Ok(r)
    }
    async fn update_rocksdb_smt(
        &self,
        opt: Opt,
        smt_name: &str,
        kvs_in: Vec<Pair>,
    ) -> Result<Response, Error> {
        let (get_root, get_proof) = (opt.get_root, opt.get_proof);

        info!("update smt in the database start: smt_name = {}, get_root = {}, get_proof = {}, kvs_len = {}, {}",
        smt_name, get_root, get_proof, kvs_in.len(), generate_kvs_info(&kvs_in));

        debug!("{}", generate_kvs_debug(&kvs_in));

        info!("create transaction ");
        let tx = self.db.transaction_default();

        info!("get handle of smt store: {}", smt_name);
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

        info!("update start， keys num = {}", kvs.len());
        for chunk in kvs.chunks(CHUNK_SIZE) {
            let _ = match rocksdb_store_smt.update_all(chunk.to_vec()) {
                Ok(_) => {}
                Err(e) => {
                    error!(
                        "update smt in database failed! smt_name = {}, err = {}",
                        smt_name, &e
                    );
                    return Err(Error::Custom(e.to_string()));
                }
            };
            commit_to_database(&tx)?;
        }
        info!("update end");

        let smt_root = rocksdb_store_smt.root().into();

        info!("generate proof");
        let smt_proofs = if !get_proof {
            default_merkel_proof()
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

        debug!("{}", generate_response_debug(&r));
        info!("update smt in the database end");
        Ok(r)
    }

    async fn update_rocksdb_smt_sequence(
        &self,
        opt: Opt,
        smt_name: &str,
        kvs_in: Vec<Pair>,
    ) -> Result<ResponseSequence, Error> {
        let (get_root, get_proof) = (opt.get_root, opt.get_proof);

        info!("update smt in the database in order start: smt_name = {}, get_root = {}, get_proof = {}, kvs_len = {}, {}",
        smt_name, get_root, get_proof, kvs_in.len(), generate_kvs_info(&kvs_in));

        let kvs_len = kvs_in.len();

        debug!("{}", generate_kvs_debug(&kvs_in));

        info!("create transaction ");
        let tx = self.db.transaction_default();

        debug!("get handle of smt store: {}", smt_name);
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
                    Ok(_) => {},
                    Err(e) => {
                        let err_str = format!(
                            "cannot update smt, err = {}, key = {}, value = {}",
                            e.to_string(),
                            slice_to_hex_string(&k.as_slice()),
                            slice_to_hex_string(&v.as_ref())
                        );
                        error!("{}", err_str,);
                        return Err(Error::Custom(err_str));
                    }
                }
            }
            let smt_root = rocksdb_store_smt.root();

            let compiled_proof = if !get_proof {
                Ok(get_empty_compiled_proof())
            } else {
                let mut vec = vec![];
                vec.push(k);

                match rocksdb_store_smt.merkle_proof(vec.clone()) {
                    Ok(p) => p.clone().compile(vec),
                    Err(e) => {
                        let err = format!(
                            "cannot generate proof for key = {}, err = {}",
                            slice_to_hex_string(k.as_slice()),
                            e.to_string()
                        );
                        error!("{}", err);
                        return Err(Error::Custom(err));
                    }
                }
            };

            match compiled_proof {
                Ok(cp) => {
                    let root = slice_to_hex_string(smt_root.as_slice());
                    let proof = slice_to_hex_string(cp.0.as_slice());
                    hashmap_roots.insert(slice_to_hex_string(k.as_slice()), root);
                    hashmap_proofs.insert(slice_to_hex_string(k.as_slice()), proof);
                }
                Err(e) => {
                    let err_str = format!(
                        "cannot generate compiled proof for key = {}, err = {}",
                        slice_to_hex_string(&k.as_slice()),
                        e.to_string()
                    );
                    error!("{}", err_str);
                    return Err(Error::Custom(err_str));
                }
            }
        } //end    for (k, v) in kvs {

        commit_to_database(&tx)?;

        if hashmap_proofs.len() != kvs_len {
            let err_str = "some keys cannot generate proof";
            error!("{}", err_str);
            return Err(Error::Custom(err_str.to_string()));
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
        debug!("{}", generate_response_sequence_debug(&r));

        info!("update smt in the database in order end");
        Ok(r)
    }
    async fn get_smt_root(&self, smt_name: &str) -> Result<SmtRoot, Error> {
        info!("get smt root of {}", smt_name);
        let snapshot = self.db.snapshot();
        let rocksdb_store_smt =
            match DefaultStoreMultiSMT::new_with_store(DefaultStoreMultiTree::<_, ()>::new(
                smt_name.as_bytes(),
                &snapshot,
            )) {
                Ok(r) => r,
                Err(e) => {
                    error!(
                        "cannot initialize database store, smt_tree = {}, err = {}",
                        smt_name, &e
                    );
                    return Err(Error::Custom(e.to_string()));
                }
            };

        let smt_root: SmtRoot = rocksdb_store_smt.root().into();
        info!(
            "get smt root end, root = {}",
            slice_to_hex_string(&smt_root.0)
        );
        Ok(smt_root)
    }

    async fn delete_smt(&self, smt_name: &str) -> Result<bool, Error> {
        info!("delete smt tree {} start", &smt_name);
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

        debug!("get handle of smt store");
        let mut rocksdb_store_smt = match DefaultStoreMultiSMT::new_with_store(
            DefaultStoreMultiTree::new(smt_name.as_bytes(), &tx),
        ) {
            Ok(r) => r,
            Err(e) => {
                error!("cannot initialize database store : {}", &e);
                return Err(Error::Custom(e.to_string()));
            }
        };

        info!("delete start, keys num = {}", kvs.len());
        let delete_chunk_size = CHUNK_SIZE;
        for chunk in kvs.chunks(delete_chunk_size) {
            let _ = match rocksdb_store_smt.update_all(chunk.to_vec()) {
                Ok(_) => {}
                Err(e) => {
                    error!("delete smt in database failed! : {}", &e);
                    return Err(Error::Custom(e.to_string()));
                }
            };
            commit_to_database(&tx)?;
        }

        info!("delete smt tree {} end", &smt_name);

        let smt_root = rocksdb_store_smt.root();
        if smt_root.eq(&H256::zero()) {
            info!("delete smt tree {}: success", &smt_name);
        } else {
            let err_str = format!("delete smt tree {}", smt_name);
            error!("{}", err_str);
            return Err(Error::Custom(err_str));
        }

        // let mut builder = ObjectParams::new();
        // builder.insert("delete_result", true);

        Ok(true)
    }
}

fn commit_to_database(tx: &OptimisticTransaction) -> Result<(), Error> {
    let _ = match tx.commit() {
        Ok(_) => {
            info!("database commit success");
        }
        Err(e) => {
            error!("database commit failed : {}", &e);
            return Err(Error::Custom(e.to_string()));
        }
    };
    Ok(())
}

fn generate_pair_string(p: &Pair) -> String {
    format!(
        "{{ key = {}, value = {}}}",
        slice_to_hex_string(&p.key.0),
        slice_to_hex_string(&p.value.as_ref())
    )
}
fn generate_kvs_info(kvs: &Vec<Pair>) -> String {
    if kvs.is_empty() {
        return String::new();
    }
    let first_kv = kvs.first().unwrap();
    let last_kv = kvs.last().unwrap();

    format!(
        "the first key-value pair is {}, the last key-value pair is {}",
        generate_pair_string(first_kv),
        generate_pair_string(last_kv)
    )
}

fn generate_kvs_debug(kvs: &Vec<Pair>) -> String {
    if kvs.is_empty() {
        return String::new();
    }

    let mut kvs_str = String::new();
    let mut count = 0;
    for p in kvs {
        kvs_str.push_str(format!("kv[{}] = {}, ", count, generate_pair_string(p)).as_str());
        count += 1;
        if count > MAX_DISPLAY_NUMS {
            break;
        }
    }
    kvs_str
}
fn generate_response_debug(response: &Response) -> String {
    let root_str = slice_to_hex_string(&response.root.0);
    let mut proofs_str = String::new();
    let mut count = 0;
    for (k, v) in &response.proofs {
        proofs_str.push_str(format!("kp{} = {{ key = {}, proof = {}}},", count, k, v).as_str());
        count += 1;
        if count > MAX_DISPLAY_NUMS {
            break;
        }
    }
    format!(
        "response: {{ root = {}, proofs =  {},}}",
        root_str, proofs_str
    )
}

fn generate_response_sequence_debug(rs: &ResponseSequence) -> String {
    let mut count = 0;

    let mut roots_str = String::new();
    for (k, v) in &rs.roots {
        roots_str.push_str(format!("{{ key = {}, root = {}}},", k, v).as_str());
        count += 1;
        if count > MAX_DISPLAY_NUMS {
            break;
        }
    }

    count = 0;
    let mut proofs_str = String::new();
    for (k, v) in &rs.proofs {
        proofs_str.push_str(format!("{{ key = {}, proof = {}}},", k, v).as_str());
        count += 1;
        if count > MAX_DISPLAY_NUMS {
            break;
        }
    }

    format!(
        "response: {{roots: {}, proofs: {},}}",
        roots_str, proofs_str
    )
}

fn default_merkel_proof() -> Vec<(String, String)> {
    let mut smt_proofs = Vec::new();
    let k = slice_to_hex_string(SmtKey::default().0.as_slice());
    let v = slice_to_hex_string(SmtProof::default().0.as_slice());
    smt_proofs.push((k, v));
    smt_proofs
}
