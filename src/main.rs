#![feature(associated_type_defaults)]


#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate rmp;
extern crate rmp_serde as rmps;


extern crate futures;
extern crate tokio_core;
extern crate tokio_io;


mod service;


fn main() {
    println!("Hello, world!");
    let conf = "/home/yuanpeixuan/workspace/tempdfs/bizur.toml";
    
    service::start_framework::<service::bizur::BizurService>(conf);
}
