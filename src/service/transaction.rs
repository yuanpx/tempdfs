extern crate serde;

pub struct Transaction {
    op: super::io_operation::IoOperation, 
    log: super::replication_log::PartLogEntry,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct ReplicaTransaction {
    op: super::io_operation::IoOperation, 
    log: super::replication_log::PartLogEntry,
}


