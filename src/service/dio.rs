extern crate futures;
use std::sync::mpsc;
use std::net::Ipv4Addr;
use std::collections::HashMap;
use std::fs::File;
use futures::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use std::io::prelude::*;
type IdType = u32;

pub enum DioCmd {
    Read(ReadOp),
    Write(IdType),
    Del(IdType),
    Append(IdType),
    End,
}

pub struct ReadOp {
    pub file_name: String,
    pub tx: UnboundedSender<Vec<u8>>,
}

pub struct JobContext {
    file: File,
}

pub fn dio_worker_thread(rx: mpsc::Receiver<DioCmd>) {
    let mut thread_job = HashMap::new();
    let mut continue_flag = true;

    while continue_flag {
        let mut dio_cmd = rx.recv().unwrap();
        if let DioCmd::End = dio_cmd {
            continue_flag = false;
        } else {
            handle_dio_cmd(dio_cmd, &mut thread_job);
        }
    }
}


pub fn handle_dio_cmd(dio_cmd: DioCmd, jobs: &mut HashMap<IdType, JobContext>) {
    
}

pub fn handle_read_file(read_op: &mut ReadOp) {
    const BUFF_LEN: usize = 1000;
    let mut file = File::open(&read_op.file_name).unwrap();
    let mut continue_flag = true;
    while continue_flag {
        let mut buf: Vec<u8> = vec![0;BUFF_LEN];
        let read_len = file.read(&mut buf[..]).unwrap();


    }
}
