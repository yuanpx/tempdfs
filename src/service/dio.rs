extern crate futures;
use std::sync::mpsc;
use std::collections::HashMap;
use std::fs::File;
use futures::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use std::io::prelude::*;
type IdType = u32;

use super::handler::Event;
use super::handler;
use tokio_core::reactor::Handle;
use std::net::SocketAddr;
use std::rc::Rc;
use std::cell::RefCell;

pub type DioLoopCmdSender = futures::sync::mpsc::UnboundedSender<()>;
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

#[derive(Serialize, Deserialize)]
struct FileBuffer{
    buf_type: u8, //1:part, 2.end
    file_buf: Vec<u8>
}

impl super::handler::Event for FileBuffer {
    fn event_id() -> u32 {
        3
    }
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
        let file_buf = if read_len < BUFF_LEN {
            continue_flag = false;
            buf.split_off(read_len);
            FileBuffer {
                buf_type: 2,
                file_buf: buf,
            }
        } else {
           FileBuffer {
                buf_type: 1,
                file_buf: buf,
           }
        }; 

        let msg = handler::gen_message(&file_buf);
        read_op.tx.send(msg).unwrap();
    }
}

pub struct DioService {
    dio_tx: mpsc::Sender<DioCmd>,
    connections: HashMap<SocketAddr, super::NioSender>,
    event_handle: Handle,
    loop_cmd_sender: DioLoopCmdSender,
}

impl super::FrameWork for DioService {
    type LoopCmd = ();

    fn new (loop_cmd_sender: futures::sync::mpsc::UnboundedSender<Self::LoopCmd>, loop_handle: Handle) -> Self {
        
        let (dtx, drx) = mpsc::channel();
        DioService {
            dio_tx: dtx,
            connections: HashMap::new(),
            event_handle: loop_handle,
            loop_cmd_sender: loop_cmd_sender,
        }
    }

    fn main_listen_addr(&self) -> &str {
        "127.0.0.1:8099"
    }

    fn handle_connect(service: Rc<RefCell<Self>>, addr: &SocketAddr, nio_sender: super::NioSender) {
        service.borrow_mut().connections.insert(addr.clone(), nio_sender);
    }

    fn handle_close(service: Rc<RefCell<Self>>, addr: &SocketAddr) {
        service.borrow_mut().connections.remove(addr);
    }

    fn handle_con_event(service: Rc<RefCell<Self>>, addr: &SocketAddr, id : super::IdType, buf: &[u8]) {
        let read_file_op: ReadFileOp = handler::gen_obj(buf);

        let tx = service.borrow_mut().connections.get_mut(addr).unwrap().clone();

        let mut read_op = ReadOp {
            file_name: read_file_op.file_name,
            tx: tx,
        };
        
        service.borrow_mut().dio_tx.send(DioCmd::Read(read_op)).unwrap();
    }

    fn handle_loop_event(service: Rc<RefCell<Self>>, cmd: Self::LoopCmd) {
        
    }
}

#[derive(Serialize, Deserialize)]
struct ReadFileOp {
    file_name: String,
}

impl handler::Event for ReadFileOp {
    fn event_id() -> u32 {
        2
    }
}
