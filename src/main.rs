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
    //let conf = env::args().nth(1).unwrap_or("/home/yuanpeixuan/workspace/tempdfs/bizur.toml".to_string());
    let mut args = env::args();

        let mode = args.nth(1).unwrap_or("client".to_string());
    match mode.as_str() {
        "proxy" => {
            //let proxy_addr = args.nth(0).unwrap();
            //let osd_addr = args.nth(0).unwrap();
            
            println!("Start Proxy!");
            info!("Start Proxy!");
            let proxy_addr = "127.0.0.1:9101".to_string();
            let osd_addr = "127.0.0.1:9102".to_string();
         
            let params = vec![proxy_addr, osd_addr];
            service::start_framework::<service::proxy::ProxyService>(&params);
        },
        "osd" => {
            println!("Start Osd!");
            info!("Start Osd!");
            let osd_addr = "127.0.0.1:9102".to_string();
            let params = vec![osd_addr];
            service::start_framework::<service::osd::OsdService>(&params);
        },
        _ => {
            println!("Start Client!");
            info!("Start Client!");
            let proxy_addr = "127.0.0.1:9101".to_string();
            let params = vec![proxy_addr];
            service::start_framework::<service::client::ClientService>(&params);
        }
    }
}
