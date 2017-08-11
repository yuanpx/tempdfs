extern crate serde;

#[derive(Serialize, Deserialize, Debug)]
pub enum IoOperation {
    Write(String, Vec<u8>),
    Del(String),
}

impl IoOperation {
    pub fn clone(&self) -> IoOperation {
        match self {
            &IoOperation::Write(ref a, ref b) => {
                IoOperation::Write(a.clone(), b.clone())
            },
            &IoOperation::Del(ref a) => {
                IoOperation::Del(a.clone())
            }
        }   
    }
}



