extern crate futures;

use std::sync::mpsc;
use std::net::Ipv4Addr;
use std::collections::HashMap;
use std::fs::File;
use futures::sync::mpsc::{UnboundedReceiver, UnboundedSender};



pub enum DioCmd {
    Read(Ipv4Addr, ReadOp),
    Write(Ipv4Addr),
    Del(Ipv4Addr),
    Append(Ipv4Addr),
    End
}

pub struct ReadOp {
    file_name: String,
    tx: UnboundedSender<Vec<u8>>,
}
pub struct JobContext {
    file: File,
}





pub fn dio_worker_thread(rx: mpsc::Receiver<DioCmd>) {
    let mut thread_job = HashMap<Ipv4Addr, Job>::new();
    let mut continue = true;

    while continue {
        let mut dio_cmd = rx.recv().unwrap();
        if let DioCmd::End = dio_cmd {
            continue = false;
        } else {
            handle_dio_cmd(dio_cmd, &mut thread_job);
        }
    }
}


pub fn handle_dio_cmd(dio_cmd: DioCmd, jobs: &mut HashMap<Ipv4Addr, Job>) {
    
}

pub fn handle_read_file(read_op: &mut ReadOp) {
    
}
