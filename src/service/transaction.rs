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
#[derive(Serialize, Deserialize, Debug)]
pub struct ReplicaTransactionResp {
    part_id: usize,
    version: usize,
}

impl Transaction {
    pub fn gen_replica_transaction(&self) -> ReplicaTransaction {
        ReplicaTransaction {
            op: self.op.clone(),
            log: self.log.clone(),
        }
    }
}


