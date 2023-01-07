use crate::blake2b::Blake2bHasherCustom;
use crate::default_store::DefaultStoreMultiTree;
use rocksdb::DBVector;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use sparse_merkle_tree::default_store::DefaultStore;
use sparse_merkle_tree::traits::Value;
use sparse_merkle_tree::{SparseMerkleTree, H256};
use std::collections::HashMap;

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SmtKey(#[serde_as(as = "serde_with::hex::Hex")] pub(crate) [u8; 32]);

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SmtValue(#[serde_as(as = "serde_with::hex::Hex")] [u8; 32]);

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SmtRoot(#[serde_as(as = "serde_with::hex::Hex")] [u8; 32]);

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SmtProof(#[serde_as(as = "serde_with::hex::Hex")] pub(crate) Vec<u8>);

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Pair {
    pub(crate) key: SmtKey,
    pub(crate) value: SmtValue,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Opt {
    pub(crate) get_proof: bool,
    pub(crate) get_root: bool,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub(crate) root: SmtRoot,
    pub(crate) proofs: HashMap<String, String>,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseSequence {
    pub(crate) roots: HashMap<String, String>,
    pub(crate) proofs: HashMap<String, String>,
}

pub type MemoryStoreSMT = SparseMerkleTree<Blake2bHasherCustom, SmtValue, DefaultStore<SmtValue>>;

pub(crate) type DefaultStoreMultiSMT<'a, T, W> =
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
