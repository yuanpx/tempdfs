extern crate futures;
extern crate serde;

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
use super::transaction::Transaction;
use super::transaction::ReplicaTransaction;
use super::transaction::ReplicaTransactionResp;
use super::replication_log::PartLogManager;
use futures::Future;
use std::io;

use super::part::*;


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





struct ProxyServerManager {
   id_proxy_clients: HashMap<usize, Client>,
   id_manager: super::IdManager,
   osd_manager: Weak<RefCell<OsdServerManager>>,
   part_manager: HashMap<usize, Part>,
   loop_handle: super::Handle,
}

impl NetEvent for ProxyServerManager {
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

    fn handle_conn_event(service: Rc<RefCell<Self>>, addr: &SocketAddr, id: usize, event_id: IdType, buf: Vec<u8>) {
        if event_id == protocol::BEGIN_SEND_FILE::event_id() {
            let req: protocol::BEGIN_SEND_FILE = super::handler::gen_obj(&buf[..]);
            let mut service_borrrow = service.borrow_mut();
            let service_mut = service_borrrow.deref_mut();
            let client = service_mut.get_proxy_client_by_id_mut(id);
            client.handle_begin_send_file(req);
            
        } else if event_id == protocol::SEND_FILE_BUFFER::event_id() {
            let req: protocol::SEND_FILE_BUFFER = super::handler::gen_obj(&buf[..]);
            let mut service_borrrow = service.borrow_mut();
            let service_mut = service_borrrow.deref_mut();
            let client = service_mut.get_proxy_client_by_id_mut(id);
            client.handle_send_file_buffer(req);
        }
    }
}

impl ProxyServerManager {
    fn new(handle: super::Handle) -> ProxyServerManager {
        ProxyServerManager {
            id_proxy_clients: HashMap::new(),
            id_manager: super::IdManager::new(),
            osd_manager: Weak::new(),
            part_manager: HashMap::new(),
            loop_handle: handle,
        }
    }

    fn get_proxy_client_by_id_mut(&mut self,id: usize) -> &mut Client {
        self.id_proxy_clients.get_mut(&id).unwrap()
    } 
}

impl ProxyServerManager {
    fn handle_part_transaction(service: &Rc<RefCell<Self>>,part_ids: &Vec<usize>, proxy_id: usize, transaction: Transaction) {
        let worker_id = trans_id_from_part_to_worker(part_ids[0]);
        let service_inner = service.clone();
        let osd_manager = service.borrow_mut().deref_mut().osd_manager.upgrade().unwrap();
        {
            let mut all_reqs = Vec::new();
            for replica_id in &part_ids[1..] {
                let replica_transaction = transaction.gen_replica_transaction();
                let addr = trans_id_from_part_to_addr(*replica_id);
                let osd_rpc_conn = osd_manager.borrow_mut().deref_mut().get_osd_rpc_conn(&addr);
                let (tx, rx) = futures::sync::oneshot::channel::<()>();
                osd_rpc_conn.borrow_mut().sync_call(replica_transaction,move |id: usize, event_id:u32 , resp: ReplicaTransactionResp| {
                    tx.send(()).unwrap();
                } );
                let req = rx.map(|_|{}).map_err(|_|{});
                all_reqs.push(req.boxed());
            }
            let mut proxy_manager = service.borrow_mut();
            let mut proxy_manager = proxy_manager.deref_mut();
            let mut part = proxy_manager.part_manager.get_mut(&worker_id).unwrap();
            let (tx, rx) = futures::sync::oneshot::channel::<()>();

            let part_io_cmd = PartIoCmd(transaction, tx);
            part.tx.send(part_io_cmd).unwrap();
            let req = rx.map(|_|{}).map_err(|_|{});
            all_reqs.push(req.boxed());
            let replicat_req = futures::future::join_all(all_reqs).map(move |_|{
                let res = super::protocol::SEND_FILE_RES{
                    res: true,
                };
                service_inner.borrow_mut().response_proxy(proxy_id, &res);
            }).map_err(|_|{});
            service.borrow_mut().deref_mut().loop_handle.spawn(replicat_req);
            
        }
    }

    fn response_proxy<T: super::handler::Event + serde::Serialize>(&mut self, proxy_id: usize, resp: &T) -> io::Result<()> {
        let mut client  = self.id_proxy_clients.get_mut(&proxy_id).unwrap();
        super::send_req(&client.sender, resp);
        Ok(())
    }
}

fn trans_id_from_part_to_worker(part_id: usize) -> usize {
    0
}

fn trans_id_from_part_to_addr(part_id: usize) -> SocketAddr {
    "127.0.0.1:9090".parse().unwrap()
}







pub struct OsdService {
    cmd_sender: futures::sync::mpsc::UnboundedSender<()>
}


impl super::FrameWork for OsdService {
    type LoopCmd = ();
    fn new(params: &Vec<String>, loop_cmd_sender: futures::sync::mpsc::UnboundedSender<Self::LoopCmd>, loop_handle: super::Handle) -> Self {
        let proxy_handle = loop_handle.clone();
        let proxy_server_manager = Rc::new(RefCell::new(ProxyServerManager::new(proxy_handle)));

        let osd_listen_addr = &params[0];
        let osd_listen_addr = osd_listen_addr.clone().parse().unwrap();
        let proxy_handle = loop_handle.clone();
        super::start_listen(proxy_server_manager, proxy_handle, osd_listen_addr);

        let other_osd_listen_addr = &params[1];
        let other_osd_listen_addr = other_osd_listen_addr.clone().parse().unwrap();

        let osd_handle = loop_handle.clone();
        let osd_server_manager = Rc::new(RefCell::new(OsdServerManager::new(loop_handle)));
        super::start_connect(osd_server_manager, osd_handle, other_osd_listen_addr);

        OsdService {
            cmd_sender: loop_cmd_sender
        }
    }

    fn handle_loop_event(service: Rc<RefCell<Self>>, cmd: Self::LoopCmd) {
        
    }
}


struct OsdServerManager {
    id_manager: super::IdManager,
    id_osds: HashMap<usize, Rc<RefCell<RpcConn<OsdServerManager>>>>,
    addr_osds: HashMap<SocketAddr, HashSet<usize>>,
    proxy_manager: Weak<RefCell<ProxyServerManager>>,
    loop_handle: super::Handle,
}

impl OsdServerManager {
    fn new(handle: super::Handle) -> OsdServerManager {
        OsdServerManager {
            id_manager: super::IdManager::new(),
            id_osds: HashMap::new(),
            addr_osds: HashMap::new(),
            proxy_manager: Weak::new(),
            loop_handle: handle,
        }
    } 

    fn get_osd_rpc_conn(&self, addr: &SocketAddr) -> Rc<RefCell<RpcConn<OsdServerManager>>> {
        let ids = self.addr_osds.get(addr).unwrap();
        let mut id: usize = 0;
        for conn_id in ids {
            id = *conn_id;
            break;
        }
        self.id_osds.get(&id).unwrap().clone()
    }
}

impl NetEvent for OsdServerManager {
    fn gen_next_id(&mut self) -> usize {
        self.id_manager.get_next_id()
    }

    fn handle_connect(service: Rc<RefCell<Self>> ,addr: &SocketAddr, id: usize, nio_sender: NioSender) {
        let rpc_con = RpcConn::new(id, nio_sender, Rc::downgrade(&service));
        let mut osd_manager = service.borrow_mut();
        osd_manager.deref_mut().id_osds.insert(id, Rc::new(RefCell::new(rpc_con)));
        let addr_entry = osd_manager.deref_mut().addr_osds.entry(addr.clone()).or_insert(HashSet::new());
        (*addr_entry).insert(id);
    }

    fn handle_close(service: Rc<RefCell<Self>>, addr: &SocketAddr, id: usize) {
        let mut osd_manager = service.borrow_mut();
        osd_manager.deref_mut().id_osds.remove(&id);
        let addr_entry = osd_manager.deref_mut().addr_osds.entry(addr.clone()).or_insert(HashSet::new());
        (*addr_entry).remove(&id);

        let handle = service.borrow_mut().deref_mut().loop_handle.clone();
        super::start_connect(service.clone(), handle, addr.clone());
    }

    fn handle_conn_event(service: Rc<RefCell<Self>>, addr: &SocketAddr, id: usize, event_id: IdType, buf: Vec<u8>) {
        
    }
    
}
