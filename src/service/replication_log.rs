extern crate rocksdb;
extern crate serde;

use std::collections::VecDeque;
use std::collections::HashMap;
use std::collections::LinkedList;

use self::rocksdb::{DB, Direction, IteratorMode};

#[derive(Serialize, Deserialize, Debug)]
struct PartLogEntry {
    op: usize,
    object: String,
    version: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct PartInfo {
    last_complete: usize,
    last_updata: usize,
    entry_list: VecDeque<PartLogEntry>,
    version: usize
}

#[derive(Serialize, Deserialize, Debug)]
struct PartLogManager {
    part_log_list: LinkedList<usize>
}

trait DBObj: serde::Serialize + serde::de::DeserializeOwned{
    fn new() -> Self;
}

impl PartLogManager {
    fn new() -> PartLogManager {
        PartLogManager {
            part_log_list: LinkedList::new()
        }
    }
}

fn get_range_db_obj<T: DBObj>(db: &mut DB, start: &str, end: &str) -> VecDeque<T> {
    
    let iter = db.iterator(IteratorMode::From(start.as_bytes(), Direction::Forward));
    let mut res = VecDeque::new();
    for (key, value) in iter {
        let buffer = value.to_vec();
        let item : T = super::handler::gen_obj(&buffer[..]);
        res.push_back(item);
        if &key.to_vec()[..] == end.as_bytes() {
           break; 
        }
    }

    res
}


fn get_db_obj<T: DBObj>(db: &mut DB, name: &str)-> Result<T, rocksdb::Error> {
    match db.get(name.as_bytes()) {
        Ok(Some(value)) => {
            let buffer = value.to_vec();
            let obj: T = super::handler::gen_obj(&buffer[..]);
            Ok(obj)
        },
        Ok(None ) => {
            Ok(T::new())
        },
        Err(e) => {
            Err(e)
        }
    }
}

fn save_db_obj<T: DBObj>(db: &mut DB, name: &str, obj: &T)  {
    let buffer = super::handler::gen_buffer(obj);
    db.put(b"part_log", &buffer[..]);
}








