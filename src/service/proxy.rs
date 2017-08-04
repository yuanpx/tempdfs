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

struct OsdChecker {
    alive_osds: HashSet<SocketAddr>,
    all_osds: HashSet<SocketAddr>,
    
}

impl OsdChecker {
    fn new() -> OsdChecker {
        OsdChecker {
            alive_osds: HashSet::new(),
            all_osds: HashSet::new(),
        }
    }

    fn is_osd_alive(&self, addr: &SocketAddr) -> bool {
        self.alive_osds.contains(addr)
    }
}




impl NetEvent for OsdChecker {

    fn gen_next_id(&mut self) -> usize {
        0
    }

    fn handle_connect(service: Rc<RefCell<Self>> ,addr: &SocketAddr, id: usize, nio_sender: NioSender) {
        service.borrow_mut().alive_osds.insert(addr.clone());
    }

    fn handle_close(service: Rc<RefCell<Self>>, addr: &SocketAddr, id: usize) {
        service.borrow_mut().alive_osds.remove(addr);
    }

    fn handle_conn_event(service: Rc<RefCell<Self>>, addr: &SocketAddr, id: usize, event_id: IdType, buf: &[u8]) {
        
    }
}

struct OsdManager {
   id_manager: super::IdManager,
   id_osds: HashMap<usize, Rc<RefCell<RpcConn<OsdManager>>>>,
   addr_osds: HashMap<SocketAddr, HashSet<usize>>,
   test_idx: usize,
   client_manager: Weak<RefCell<ClientManager>>,
}
impl OsdManager {
    fn get_osd_by_id_mut(&mut self, id: usize) -> &mut Rc<RefCell<RpcConn<OsdManager>>> {
        self.id_osds.get_mut(&id).unwrap()
    }
}



struct ClientManager {
    id_manager: super::IdManager,
    id_clients: HashMap<usize, NioSender>,
    osd_manager: Weak<RefCell<OsdManager>>,
}

impl ClientManager {
    fn get_client_by_id_mut(&mut self, id: usize) -> &mut NioSender {
        self.id_clients.get_mut(&id).unwrap()
    }
}


struct Proxy {
   client_manager: Rc<RefCell<ClientManager>>,
   osd_manager: Rc<RefCell<OsdManager>>,
}

impl NetEvent for ClientManager {

    fn gen_next_id(&mut self) -> usize {
        self.id_manager.get_next_id()
    }

    fn handle_connect(service: Rc<RefCell<Self>> ,addr: &SocketAddr, id: usize, nio_sender: NioSender) {
        service.borrow_mut().id_clients.insert(id, nio_sender);
    }

    fn handle_close(service: Rc<RefCell<Self>>, addr: &SocketAddr, id: usize) {
        service.borrow_mut().id_clients.remove(&id);
    }

    fn handle_conn_event(service: Rc<RefCell<Self>>, addr: &SocketAddr, id: usize, event_id: IdType, buf: &[u8]) {
        if event_id == protocol::BEGIN_SEND_FILE::event_id() {
           let req: protocol::BEGIN_SEND_FILE = super::handler::gen_obj(buf);
           service.borrow_mut().handle_begin_send_file(req);
            
        } else if event_id == protocol::SEND_FILE_BUFFER::event_id() {
            let req: protocol::SEND_FILE_BUFFER = super::handler::gen_obj(buf);
            service.borrow_mut().handle_send_file_buffer(req);
        }
    }
}

impl ClientManager {
    fn handle_begin_send_file(&mut self, req: protocol::BEGIN_SEND_FILE) {
       let osd_manager: Rc<RefCell<OsdManager>> = self.osd_manager.upgrade().unwrap();
       let test_idx = osd_manager.borrow_mut().deref_mut().test_idx;
       let rpc_conn = osd_manager.borrow_mut().get_osd_by_id_mut(test_idx).clone();
       rpc_conn.borrow_mut().async_call(req);
    }

    fn handle_send_file_buffer(&mut self, req: protocol::SEND_FILE_BUFFER) {
        let osd_manager: Rc<RefCell<OsdManager>> = self.osd_manager.upgrade().unwrap();
        let test_idx = osd_manager.borrow_mut().deref_mut().test_idx;
        let rpc_conn = osd_manager.borrow_mut().get_osd_by_id_mut(test_idx).clone();
        rpc_conn.borrow_mut().async_call(req);
    }
}











impl NetEvent for OsdManager {
    fn gen_next_id(&mut self) -> usize {
        self.id_manager.get_next_id()
    }

    fn handle_connect(service: Rc<RefCell<Self>> ,addr: &SocketAddr, id: usize, nio_sender: NioSender) {
        let rpc_con = RpcConn::new(id, nio_sender, Rc::downgrade(&service));
        let mut osd_manager = service.borrow_mut();
        osd_manager.deref_mut().test_idx = id;
        osd_manager.deref_mut().id_osds.insert(id, Rc::new(RefCell::new(rpc_con)));
        let addr_entry = osd_manager.deref_mut().addr_osds.entry(addr.clone()).or_insert(HashSet::new());
        (*addr_entry).insert(id);
    }

    fn handle_close(service: Rc<RefCell<Self>>, addr: &SocketAddr, id: usize) {
        let mut osd_manager = service.borrow_mut();
        osd_manager.deref_mut().id_osds.remove(&id);
        let addr_entry = osd_manager.deref_mut().addr_osds.entry(addr.clone()).or_insert(HashSet::new());
        (*addr_entry).remove(&id);
    }

    fn handle_conn_event(service: Rc<RefCell<Self>>, addr: &SocketAddr, id: usize, event_id: IdType, buf: &[u8]) {
        
    }
    
}


