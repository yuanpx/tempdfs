extern crate toml;
extern crate serde;

use std::fs::File;
use std::io::prelude::*;
use serde::Deserialize;
use std::clone::Clone;
use std::io::Result;

#[derive(Debug, Deserialize)]
pub struct BizurConfig {
    pub addrs: Vec<String>,
    pub heartbeat_timeout: u64,
    pub req_timeout: u64,
}

impl Clone for BizurConfig {
    fn clone(&self) -> Self {
        BizurConfig {
            addrs: self.addrs.clone(),
            heartbeat_timeout: self.heartbeat_timeout.clone(),
            req_timeout: self.req_timeout.clone(),
        }
        
    }
    
}


pub fn load_config(path: &str) -> Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}




