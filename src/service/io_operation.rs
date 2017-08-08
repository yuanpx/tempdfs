extern crate serde;

#[derive(Serialize, Deserialize, Debug)]
pub enum IoOperation {
    Write(String, Vec<u8>),
    Del(String),
}



