extern crate serde;

use super::handler::Event;

#[derive(Serialize, Deserialize, Debug)]
pub struct BEGIN_SEND_FILE{
    pub name: String,
}

impl Event for BEGIN_SEND_FILE {
    fn event_id() -> u32 {
        1
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SEND_FILE_BUFFER {
    pub more: bool,
    pub buf: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SEND_FILE_RES {
    pub res: bool,
}



impl Event for SEND_FILE_BUFFER {
    fn event_id() -> u32 {
       2 
    }
}

impl Event for super::transaction::ReplicaTransaction {
    fn event_id() -> u32 {
        3
    }
}

impl Event for super::transaction::ReplicaTransactionResp {
    fn event_id() -> u32 {
        4
    }
}

impl Event for SEND_FILE_RES {
    fn event_id() -> u32 {
        5
    }
}
