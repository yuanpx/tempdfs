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

impl Event for SEND_FILE_BUFFER {
    fn event_id() -> u32 {
       2 
    }
}




#[derive(Serialize, Deserialize, Debug)]
pub struct REPLICATION_BEGIN_SEND_FILE{
    pub name: String,
}

impl Event for REPLICATION_BEGIN_SEND_FILE {
    fn event_id() -> u32 {
        1
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct REPLICATION_SEND_FILE_BUFFER {
    pub more: bool,
    pub buf: Vec<u8>,
}

impl Event for REPLICATION_SEND_FILE_BUFFER {
    fn event_id() -> u32 {
        2 
    }
}
