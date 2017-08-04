#![feature(associated_type_defaults)]

#[macro_use]
extern crate log;
extern crate env_logger; 

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate rmp;
extern crate rmp_serde as rmps;


extern crate futures;
extern crate tokio_core;
extern crate tokio_io;

use std::env;


mod service;



fn main() {
    env_logger::init();
    info!("starting up");
    let conf = env::args().nth(1).unwrap_or("/home/yuanpeixuan/workspace/tempdfs/bizur.toml".to_string());
    
    //service::start_framework::<service::bizur::BizurService>(&conf);
}
