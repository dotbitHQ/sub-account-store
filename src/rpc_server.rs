use std::collections::HashMap;
use jsonrpsee::{
    core::{async_trait, Error},
    proc_macros::rpc,
};
use rocksdb::{OptimisticTransaction, OptimisticTransactionDB};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use super::{
    default_store::DefaultStoreMultiTree,
    structures::{
        DefaultStoreMultiSMT, MemoryStoreSMT, Opt, Pair, Response, SmtKey, SmtProof, SmtRoot,
        SmtValue,
    },
    utils::slice_to_hex_string,
};
use rayon::prelude::*;
use sparse_merkle_tree::{
    traits::{StoreWriteOps, Value},
    BranchKey, H256,
};

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

    #[method(name = "get_smt_root")]
    async fn get_smt_root(&self, smt_name: &str) -> Result<SmtRoot, Error>;

    #[method(name = "delete_smt")]
    async fn delete_smt(&self, smt_name: &str) -> Result<OperationResult, Error>;
}

#[async_trait]
impl RpcServer for RpcServerImpl {
    async fn build_memory_smt(
        &self,
        opt: Opt,
        _smt_name: &str,
        kvs_in: Vec<Pair>,
    ) -> Result<Response, Error> {
        let get_root = opt.get_root;
        let get_proof = opt.get_proof;

        //check if data is empty
        if kvs_in.is_empty() {
            return Err(Error::EmptyAllowList("key value vec is null"));
        }

        let kvs: Vec<(H256, SmtValue)> = kvs_in
            .clone()
            .into_iter()
            .map(|k| (k.key.0.into(), k.value))
            .collect();
        let keys: Vec<H256> = kvs_in.clone().into_iter().map(|k| k.key.0.into()).collect();

        // create memory tree
        let mut memory_store_smt = MemoryStoreSMT::new_with_store(Default::default()).unwrap();

        let smt_root = memory_store_smt
            .update_all(kvs)
            .expect("cannot create tree")
            .into();

        let smt_proofs = if !get_proof {
            let mut smt_proofs = Vec::new();
            //let k = SmtKey::default();
            let k = slice_to_hex_string(SmtKey::default().0.as_slice());
            let v = slice_to_hex_string(SmtProof::default().0.as_slice());
            smt_proofs.push((k, v));
            smt_proofs
        } else {
            keys.par_iter()
                .map(|k| {
                    let mut vec = Vec::new();
                    vec.push(*k);
                    let proof = memory_store_smt
                        .merkle_proof(vec.clone())
                        .expect("merkle_proof error");

                    let compiled_proof = proof.clone().compile(vec).expect("compile error");

                    let k = slice_to_hex_string(k.to_h256().as_slice());
                    let v = slice_to_hex_string(compiled_proof.0.as_slice());
                    (k, v)
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
        let get_root = opt.get_root;
        let get_proof = opt.get_proof;

        //Todo multi thread support
        let tx = self.db.transaction_default();
        let mut rocksdb_store_smt = DefaultStoreMultiSMT::new_with_store(
            DefaultStoreMultiTree::new(smt_name.as_bytes(), &tx),
        )
        .unwrap();
        let kvs: Vec<(H256, SmtValue)> = kvs_in
            .clone()
            .into_par_iter()
            .map(|k| (k.key.0.into(), k.value))
            .collect();
        let keys: Vec<H256> = kvs_in
            .clone()
            .into_par_iter()
            .map(|k| k.key.0.into())
            .collect();

        let smt_root = rocksdb_store_smt
            .update_all(kvs.clone().into())
            .expect("update error")
            .into();
        //Todo to check consistency
        tx.commit().expect("db commit error");

        let smt_proofs = if !get_proof {
            let mut smt_proofs = vec![];
            let k = format!("{:?}", SmtKey::default());
            let v = format!("{:?}", SmtProof::default());

            //smt_proofs.push((, SmtProof::default()));
            smt_proofs.push((k, v));
            smt_proofs
        } else {
            keys.par_iter()
                .map(|k| {
                    let mut vec = vec![];
                    vec.push(*k);
                    let proof = rocksdb_store_smt
                        .merkle_proof(vec.clone())
                        .expect("merkle_proof error");

                    let compiled_proof = proof.clone().compile(vec).expect("compile error");

                    let k = slice_to_hex_string(k.to_h256().as_slice());
                    let v = slice_to_hex_string(compiled_proof.0.as_slice());
                    (k, v)
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
    async fn get_smt_root(&self, smt_name: &str) -> Result<SmtRoot, Error> {
        let snapshot = self.db.snapshot();
        let rocksdb_store_smt = DefaultStoreMultiSMT::new_with_store(
            DefaultStoreMultiTree::<_, ()>::new(smt_name.as_bytes(), &snapshot),
        )
        .expect("cannot get smt storage");
        let smt_root = rocksdb_store_smt.root().into();
        Ok(smt_root)
    }

    async fn delete_smt(&self, smt_name: &str) -> Result<OperationResult, Error> {
        let tx = self.db.transaction_default();
        let mut rocksdb_store_smt = DefaultStoreMultiSMT::new_with_store(
            DefaultStoreMultiTree::new(smt_name.as_bytes(), &tx),
        )
        .unwrap();
        let root = rocksdb_store_smt.root();
        let branch_key = BranchKey {
            height: 255,
            node_key: *root,
        };
        let mut store = rocksdb_store_smt.store_mut();
        <DefaultStoreMultiTree<'_, OptimisticTransaction, ()> as StoreWriteOps<SmtValue>>::remove_branch(&mut store, &branch_key).expect("cannot remove branch");
        Ok(OperationResult(true))
    }


}