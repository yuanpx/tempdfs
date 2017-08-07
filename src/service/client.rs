extern crate futures;
use std::collections::{HashSet, HashMap, LinkedList};
use std::net::SocketAddr;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use super::NioSender;

use super::NetEvent;
use super::IdType;
use super::RpcConn;
use std::ops::DerefMut;
use super::protocol;
use super::handler::Event;
use std::fs::File;

    
struct Client {
    sender: Option<NioSender>
}

impl Client {
    fn new() -> Client {
        Client {
            sender: None
        }
    }
}


impl NetEvent for Client {
    fn gen_next_id(&mut self) -> usize {
        0
    }

    fn handle_connect(service: Rc<RefCell<Self>> ,addr: &SocketAddr, id: usize, nio_sender: NioSender) {
        let mut service_borrrow = service.borrow_mut();
        let service_mut = service_borrrow.deref_mut();

        let create_file_req = protocol::BEGIN_SEND_FILE {
            name: "test_file".to_string()
        };
        let buf: Vec<u8> = b"test_test".to_vec();
        let write_file_req =  protocol::SEND_FILE_BUFFER {
            more: false,
            buf: buf,
        };

        super::send_req(&nio_sender, &create_file_req);
        super::send_req(&nio_sender, &write_file_req);
        service_mut.sender = Some(nio_sender);
    }

    fn handle_close(service: Rc<RefCell<Self>>, addr: &SocketAddr, id: usize) {
        let mut service_borrrow = service.borrow_mut();
        let service_mut = service_borrrow.deref_mut();
        service_mut.sender = None;
    }

    fn handle_conn_event(service: Rc<RefCell<Self>>, addr: &SocketAddr, id: usize, event_id: IdType, buf: &[u8]) {
    }
}


pub struct ClientService {
    cmd_sender: futures::sync::mpsc::UnboundedSender<()>,
}

impl super::FrameWork for ClientService {
    type LoopCmd = ();

    fn new(path: &Vec<String>, loop_cmd_sender: futures::sync::mpsc::UnboundedSender<Self::LoopCmd>, loop_handle: super::Handle) -> Self {
        let proxy_addr = &path[0];
        let proxy_addr = proxy_addr.clone().parse().unwrap();
        let client = Rc::new(RefCell::new(Client::new()));
        super::start_connect(client, loop_handle, proxy_addr);
        ClientService {
            cmd_sender: loop_cmd_sender
        }
    }

    fn handle_loop_event(service: Rc<RefCell<Self>>, cmd: Self::LoopCmd) {
        
    }



}
