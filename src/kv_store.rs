use crate::structures::get_db_path;
use rocksdb::prelude::{Delete, Get, Open, Put};
use rocksdb::DB;
use std::sync::Arc;

const VERSION_DB_SUFFIX: &str = "/version";
pub trait KVStore {
    fn init(file_path: &str) -> Self;
    fn save(&self, k: &str, v: &str) -> bool;
    fn find(&self, k: &str) -> Option<String>;
    fn delete(&self, k: &str) -> bool;
}
#[derive(Clone)]
pub struct RocksDB {
    db: Arc<DB>,
}
impl KVStore for RocksDB {
    fn init(file_path: &str) -> Self {
        RocksDB {
            db: Arc::new(DB::open_default(file_path).unwrap()),
        }
    }
    fn save(&self, k: &str, v: &str) -> bool {
        self.db.put(k.as_bytes(), v.as_bytes()).is_ok()
    }
    fn find(&self, k: &str) -> Option<String> {
        match self.db.get(k.as_bytes()) {
            Ok(Some(v)) => {
                let result = String::from_utf8(v.to_owned()).unwrap();
                println!("Finding '{}' returns '{}'", k, result);
                Some(result)
            }
            Ok(None) => {
                println!("Finding '{}' returns None", k);
                None
            }
            Err(e) => {
                println!("Error retrieving value for {}: {}", k, e);
                None
            }
        }
    }
    fn delete(&self, k: &str) -> bool {
        self.db.delete(k.as_bytes()).is_ok()
    }
}
fn get_version_db_path() -> String {
    let mut db_path = get_db_path();
    db_path.push_str(VERSION_DB_SUFFIX);
    db_path
}
pub fn get_smt_tree_name(smt_name: &str) -> String {
    //init db
    let db: RocksDB = KVStore::init(get_version_db_path().as_str());
    //get smt_name version
    match db.find(smt_name) {
        Some(v) => {
            //if existed then get
            format!("{}_{}", smt_name, v)
        }
        None => {
            //if none then insert
            let version = "0";
            db.save(smt_name, version);
            format!("{}_{}", smt_name, version)
        }
    }
}

pub fn upgrade_smt_tree_version(smt_name: &str) -> bool {
    //init db
    let db: RocksDB = KVStore::init(get_version_db_path().as_str());
    //get smt_name version
    match db.find(smt_name) {
        Some(v) => {
            //if existed then get
            match v.parse::<usize>() {
                Ok(old_version) => {
                    let new_version = old_version + 1;
                    let new_version_str = new_version.to_string();
                    db.save(smt_name, new_version_str.as_str());
                    true
                }
                Err(e) => {
                    println!("cannot parse string to number : {}", e);
                    false
                }
            }
        }
        None => {
            //if none then insert
            println!("cannot find smt tree named :{}", smt_name);
            false
        }
    }
}
