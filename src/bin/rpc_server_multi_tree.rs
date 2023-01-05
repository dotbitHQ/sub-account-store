use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;

use jsonrpsee::{
    core::{async_trait, Error},
    http_server::HttpServerBuilder,
    proc_macros::rpc,
};

use rocksdb::{prelude::Open, DBVector, OptimisticTransactionDB};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use das_smt_rpc::{blake2b::Blake2bHasherCustom, default_store::DefaultStoreMultiTree};

use rayon::prelude::*;
use sparse_merkle_tree::{default_store::DefaultStore, traits::Value, SparseMerkleTree, H256};

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SmtKey(#[serde_as(as = "serde_with::hex::Hex")] [u8; 32]);

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SmtValue(#[serde_as(as = "serde_with::hex::Hex")] [u8; 32]);

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SmtRoot(#[serde_as(as = "serde_with::hex::Hex")] [u8; 32]);

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SmtProof(#[serde_as(as = "serde_with::hex::Hex")] Vec<u8>);

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Pair {
    key: SmtKey,
    value: SmtValue,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Opt {
    get_proof: bool,
    get_root: bool,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    root: SmtRoot,
    proofs: HashMap<String, String>,
}

pub type MemoryStoreSMT = SparseMerkleTree<Blake2bHasherCustom, SmtValue, DefaultStore<SmtValue>>;

type DefaultStoreMultiSMT<'a, T, W> =
    SparseMerkleTree<Blake2bHasherCustom, SmtValue, DefaultStoreMultiTree<'a, T, W>>;





impl From<&H256> for SmtKey {
    fn from(h: &H256) -> Self {
        let mut key = [0u8; 32];
        for (i, v) in h.as_slice().iter().enumerate() {
            key[i] = *v;
        }
        SmtKey(key)
    }
}
impl Into<H256> for SmtKey {
    fn into(self) -> H256 {
        let mut smtkey = [0u8; 32];
        for (i, v) in self.0.as_slice().iter().enumerate() {
            smtkey[i] = *v;
        }
        smtkey.into()
    }
}
impl From<DBVector> for SmtValue {
    fn from(vec: DBVector) -> Self {
        SmtValue(vec.as_ref().try_into().expect("stored value is 32 bytes"))
    }
}

impl AsRef<[u8]> for SmtValue {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}


impl Value for SmtValue {
    fn to_h256(&self) -> H256 {
        self.0.into()
    }

    fn zero() -> Self {
        Self([0u8; 32])
    }
}

impl From<&H256> for SmtRoot {
    fn from(h: &H256) -> Self {
        let mut root = [0u8; 32];
        for (i, v) in h.as_slice().iter().enumerate() {
            root[i] = *v;
        }
        SmtRoot(root)
    }
}

#[inline]
fn slice_to_hex_string(slice :&[u8]) -> String
{
    let mut s = String::new();
    for i in slice {
        s.push_str(format!("{:02x}", i).as_str());
    }
    println!("hex = {}", &s);
    return s;
}


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

    #[method(name = "update_all")]
    async fn update_all(&self, tree: &str, kvs: Vec<(SmtKey, SmtValue)>) -> Result<SmtRoot, Error>;

    #[method(name = "merkle_proof")]
    async fn merkle_proof(&self, tree: &str, keys: Vec<SmtKey>) -> Result<SmtProof, Error>;
}

pub struct RpcServerImpl {
    db: OptimisticTransactionDB,
}

impl RpcServerImpl {
    fn new(db: OptimisticTransactionDB) -> Self {
        Self { db }
    }
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
        }else {
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
        let keys: Vec<H256> = kvs_in.clone().into_par_iter().map(|k| k.key.0.into()).collect();

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
                    (k, v) })
                .collect::<Vec<_>>()
        };

        let hashmap_proofs: HashMap<_, _> = smt_proofs.into_par_iter().collect();

        let r = if !get_root {
            Response {
                root: SmtRoot::default(),
                proofs: hashmap_proofs,
            }
        }else {
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

    async fn update_all(&self, tree: &str, kvs: Vec<(SmtKey, SmtValue)>) -> Result<SmtRoot, Error> {
        let kvs: Vec<(H256, SmtValue)> = kvs.into_iter().map(|(k, v)| (k.0.into(), v)).collect();


        let tx = self.db.transaction_default();
        let mut rocksdb_store_smt =
            DefaultStoreMultiSMT::new_with_store(DefaultStoreMultiTree::new(tree.as_bytes(), &tx))
                .unwrap();
        rocksdb_store_smt.update_all(kvs).expect("update_all error");

        tx.commit().expect("db commit error");
        Ok(SmtRoot(rocksdb_store_smt.root().clone().into()))
    }

    async fn merkle_proof(&self, tree: &str, keys: Vec<SmtKey>) -> Result<SmtProof, Error> {
        let keys: Vec<H256> = keys.into_iter().map(|k| k.0.into()).collect();
        let snapshot = self.db.snapshot();
        let rocksdb_store_smt = DefaultStoreMultiSMT::new_with_store(
            DefaultStoreMultiTree::<_, ()>::new(tree.as_bytes(), &snapshot),
        )
        .unwrap();
        let proof = rocksdb_store_smt
            .merkle_proof(keys.clone())
            .expect("merkle_proof error");
        Ok(SmtProof(proof.compile(keys).expect("compile error").0))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let args: Vec<String> = env::args().collect();
    let db_path = args.get(1).expect("args db_path not found");
    let listen_addr = args.get(2).expect("args listen_addr not found");
    let db = OptimisticTransactionDB::open_default(db_path).unwrap();
    let server = HttpServerBuilder::default()
        .build(listen_addr.parse::<SocketAddr>()?)
        .await?;
    let _handle = server.start(RpcServerImpl::new(db).into_rpc())?;
    println!("Server started at http://{}", listen_addr);
    futures::future::pending().await
}
