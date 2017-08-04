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
use std::io::prelude::*;
use std::io::Write;


struct TaskInfo {
   file: File, 
}

impl TaskInfo {
    fn new(file: File) -> TaskInfo {
        TaskInfo {
            file: file,
        } 
    }
}

struct Client {
   sender: NioSender, 
   task_info: Option<TaskInfo>,
}

impl Client {
    fn new(sender: NioSender) -> Client {
        Client{
            sender: sender,
            task_info: None,
        }
    } 

    fn handle_begin_send_file(&mut self, req: protocol::BEGIN_SEND_FILE) {
        let file = File::create(&req.name).unwrap();
        let task_info = TaskInfo::new(file);
        self.task_info = Some(task_info);
    }



    fn handle_send_file_buffer(&mut self, req: protocol::SEND_FILE_BUFFER){
        let mut task_info = self.task_info.take().unwrap();
        task_info.file.write_all(&req.buf[..]).unwrap();
        if req.more == false {
            task_info.file.flush().unwrap();
        } else {
            self.task_info = Some(task_info);
        }
    }
}





struct Osd {
   id_proxy_clients: HashMap<usize, Client>,
   id_manager: super::IdManager,
}

impl NetEvent for Osd {
    fn gen_next_id(&mut self) -> usize {
        self.id_manager.get_next_id()
    }

    
    fn handle_connect(service: Rc<RefCell<Self>> ,addr: &SocketAddr, id: usize, nio_sender: NioSender) {
        let client = Client::new(nio_sender);
        service.borrow_mut().id_proxy_clients.insert(id, client);
    }

    fn handle_close(service: Rc<RefCell<Self>>, addr: &SocketAddr, id: usize) {
        service.borrow_mut().id_proxy_clients.remove(&id);
    }

    fn handle_conn_event(service: Rc<RefCell<Self>>, addr: &SocketAddr, id: usize, event_id: IdType, buf: &[u8]) {
        if event_id == protocol::BEGIN_SEND_FILE::event_id() {
            let req: protocol::BEGIN_SEND_FILE = super::handler::gen_obj(buf);
            let mut service_borrrow = service.borrow_mut();
            let service_mut = service_borrrow.deref_mut();
            let client = service_mut.get_proxy_client_by_id_mut(id);
            client.handle_begin_send_file(req);
            
        } else if event_id == protocol::SEND_FILE_BUFFER::event_id() {
            let req: protocol::SEND_FILE_BUFFER = super::handler::gen_obj(buf);
            let mut service_borrrow = service.borrow_mut();
            let service_mut = service_borrrow.deref_mut();
            let client = service_mut.get_proxy_client_by_id_mut(id);
            client.handle_send_file_buffer(req);
        }
    }
}

impl Osd {
    fn get_proxy_client_by_id_mut(&mut self,id: usize) -> &mut Client {
        self.id_proxy_clients.get_mut(&id).unwrap()
    } 
}


