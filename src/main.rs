#![feature(associated_type_defaults)]


#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate rmp;
extern crate rmp_serde as rmps;


extern crate futures;
extern crate tokio_core;
extern crate tokio_io;

use std::rc::Rc;
use std::cell::RefCell;
use std::iter;
use std::env;
use std::io::{Error, BufReader};

use futures::Future;
use futures::stream::{self, Stream};
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;
use tokio_io::io;
use tokio_io::AsyncRead;
use std::sync::mpsc;
use std::net::SocketAddr;



use std::vec::Vec;

mod service;


fn main() {
    println!("Hello, world!");

    service::start_framework::<service::dio::DioService>();
}
