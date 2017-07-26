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
    pub host: String,
    pub listen_addr: String,
}



pub fn load_config(path: &str) -> Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}




