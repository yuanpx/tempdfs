extern crate serde;

#[derive(Serialize, Deserialize, Debug)]
struct Transaction {
    op: super::io_operation::IoOperation, 
    log: super::replication_log::PartLogEntry,
}


#[derive(Serialize, Deserialize, Debug)]
struct ReplicaTransaction {
    op: super::io_operation::IoOperation, 
    log: super::replication_log::PartLogEntry,
}


