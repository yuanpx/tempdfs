extern crate serde;


pub mod dio;
pub mod handler;
pub mod bizur;
pub mod bizur_conf;
pub mod http;
pub mod ring;
pub mod proxy;
pub mod osd;
pub mod client;
pub mod protocol;
pub mod replication_log;
pub mod io_operation;
pub mod transaction;
pub mod part;

pub type NioSender = futures::sync::mpsc::UnboundedSender<Vec<u8>>;
pub type IdType = u32;
extern crate futures;
extern crate tokio_core;
extern crate tokio_io;


use std::collections::{HashMap, LinkedList};
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::iter;
use std::env;
use std::io::{Error, BufReader};

use futures::Future;
use futures::stream::{self, Stream};
use tokio_core::net::{TcpListener, TcpStream};
use tokio_core::reactor::Core;
use tokio_core::reactor::Handle;
use tokio_io::io;
use tokio_io::AsyncRead;
use std::net::SocketAddr;
use std::ops::DerefMut;
use std::vec::Vec;
use std::boxed::FnBox;



pub trait FrameWork {
    type LoopCmd;
    fn new(params: &Vec<String>, loop_cmd_sender: futures::sync::mpsc::UnboundedSender<Self::LoopCmd>, loop_handle: Handle) -> Self;

    fn handle_loop_event(service: Rc<RefCell<Self>>, cmd: Self::LoopCmd);
}

pub trait NetEvent {

    fn gen_next_id(&mut self) -> usize;

    fn handle_connect(service: Rc<RefCell<Self>> ,addr: &SocketAddr, id: usize, nio_sender: NioSender);

    fn handle_close(service: Rc<RefCell<Self>>, addr: &SocketAddr, id: usize);

    fn handle_conn_event(service: Rc<RefCell<Self>>, addr: &SocketAddr, id: usize, event_id: IdType, buf: &[u8]);
}



pub fn start_framework<T: 'static + FrameWork>(params: &Vec<String>) {
    println!("Start FrameWork");
    info!("Start FrameWork");
    let (loop_cmd_tx, loop_cmd_rx) = futures::sync::mpsc::unbounded::<T::LoopCmd>();
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let loop_handler = handle.clone();
    let service = T::new(params, loop_cmd_tx, loop_handler);
    let service = Rc::new(RefCell::new(service));
    let service_loop_handler = service.clone();

    let service_cmd_handler = service_loop_handler.clone();
    let loop_cmd_handler = loop_cmd_rx.fold(service_cmd_handler, |service_cmd_handler, cmd| {
        T::handle_loop_event(service_cmd_handler.clone(), cmd);
        Ok(service_cmd_handler)
    });

    let loop_cmd_handler = loop_cmd_handler.map(|_| ());
    let loop_cmd_handler = loop_cmd_handler.map_err(|_| ());

    core.run(loop_cmd_handler).unwrap();
}


pub fn handle_connection<T: 'static + NetEvent>(service: &Rc<RefCell<T>>, handle: &Handle, addr: SocketAddr, stream: TcpStream) {
        info!("New Connection: {}", addr);
        let (reader, writer) = stream.split();
        let (tx, rx) = futures::sync::mpsc::unbounded::<Vec<u8>>();
        // connections.borrow_mut().insert(addr, tx);
        let next_id = service.borrow_mut().gen_next_id();
        T::handle_connect(service.clone(), &addr, next_id, tx);
        let service_inner = service.clone();
        let reader = BufReader::new(reader);

        let iter = stream::iter(iter::repeat(()).map(Ok::<(), Error>));
        let socket_reader = iter.fold((reader, service_inner),
                                      move |(reader, service_inner), _| {
                                          let header_buf: [u8; 4] = [0; 4];
                                          let header = io::read_exact(reader, header_buf);
                                          let body = header.and_then(|(reader, header)| {
                                              let body_len: u32 = handler::get_u32_length(&header[..]);
                                              let buff: Vec<u8> = vec![0;body_len as usize];
                                              io::read_exact(reader, buff)
                                          });

                                          let service_handle_event = service_inner.clone();
                                          body.map(move |(reader, vec)| {
                                              let event_id: u32 = handler::get_u32_length(&vec[0..4]);
                                              T::handle_conn_event(service_handle_event, &addr,next_id, event_id , &vec[4..]);
                                              (reader, service_inner)
                                          })
                                      });

        let socket_writer = rx.fold(writer, |writer, msg| {
            let amt = io::write_all(writer, msg);
            let amt = amt.map(|(writer, _)| writer);
            amt.map_err(|_| ())
        });


        // let connections_in = connections.clone();
        let service_in = service.clone();
        let socket_reader = socket_reader.map_err(|_| ());
        let connection = socket_reader.map(|_| ()).select(socket_writer.map(|_| ()));
        handle.spawn(connection.then(move |_| {
            // connections_in.borrow_mut().remove(&addr);
            T::handle_close(service_in, &addr, next_id);
            info!("Connection {} closed.", addr);
            Ok(())
        }));
}

pub fn start_listen<T: 'static + NetEvent>(service: Rc<RefCell<T>>, handle: Handle, addr: SocketAddr) {
    let socket = TcpListener::bind(&addr, &handle).unwrap();
    info!("Listening on: {}", addr);
    let out_handle = handle.clone();
    // let connections = Rc::new(RefCell::new(HashMap::new()));
    let srv = socket.incoming().for_each(move |(stream, addr)| {
        info!("New Connection: {}", addr);
        handle_connection(&service, &handle, addr, stream);
        Ok(())
    });
    let srv = srv.map(|_|()).map_err(|_|());

    out_handle.spawn(srv);
}


pub fn start_connect<T: 'static + NetEvent>(service: Rc<RefCell<T>>, handle: Handle, addr: SocketAddr) {
    info!("connect on: {}", addr);
    let out_handle = handle.clone();
    let tcp = TcpStream::connect(&addr, &handle);
    let client = tcp.map(move |stream|{
        handle_connection(&service, &handle, addr, stream);
        ()
    }).map_err(|_|());

    out_handle.spawn(client);
}

pub struct RpcConn<T> {
    id: usize,
    sender: NioSender, 
    pendding_calls: LinkedList<Box<FnBox(usize, IdType, &[u8])>>, 
    data: Weak<RefCell<T>>,
}


impl <T: 'static> RpcConn<T> {

    fn new(id: usize, sender: NioSender,data: Weak<RefCell<T>>) -> RpcConn<T> {
        RpcConn {
            id: id,
            sender: sender,
            pendding_calls: LinkedList::new(),
            data: data
        }
    }


    fn async_call<REQ: serde::Serialize + handler::Event>(&mut self, req: REQ) {
        send_req(&mut self.sender, &req);
    }

    fn sync_call<REQ: serde::Serialize + handler::Event, RESP: 'static + serde::de::DeserializeOwned, HANDLER:'static + FnOnce(usize, IdType, RESP)>(&mut self, req: REQ, resp_handler: HANDLER) {
        send_req(&mut self.sender, &req);
        let resp = gen_resp_handler(resp_handler);
        self.pendding_calls.push_back(resp);
    }

    fn handle_sync_callback(&mut self, addr: &SocketAddr, id: usize, event_id: IdType, buf: &[u8]) {
        let mut callback = self.pendding_calls.pop_front().unwrap();
        callback.call_box((id, event_id, buf));
    }
    
}



impl <T: 'static + NetEvent> NetEvent for RpcConn<T> {
    fn gen_next_id(&mut self) -> usize {
        let real_data = self.data.upgrade().unwrap();
        let id = real_data.borrow_mut().gen_next_id();
        id
    }

    fn handle_connect(service: Rc<RefCell<Self>> ,addr: &SocketAddr, id: usize, nio_sender: NioSender) {
        let real_data = service.borrow_mut().data.upgrade().unwrap();
        T::handle_connect(real_data, addr, id, nio_sender);
    }

    fn handle_close(service: Rc<RefCell<Self>>, addr: &SocketAddr, id: usize) {
        let real_data = service.borrow_mut().data.upgrade().unwrap();
        T::handle_close(real_data, addr, id);
    }

    fn handle_conn_event(service: Rc<RefCell<Self>>, addr: &SocketAddr, id: usize, event_id: IdType, buf: &[u8]) {
        service.borrow_mut().deref_mut().handle_sync_callback(addr, id, event_id, buf);
    }

}


type BuffHandler= FnBox(usize, IdType, &[u8]);

pub  fn gen_resp_handler<RESP:'static + serde::de::DeserializeOwned, HANDLER: 'static + FnOnce(usize, IdType, RESP)>(resp_handler: HANDLER) -> Box<BuffHandler>
{

    Box::new(move |id, event_id, buf| {
        let res : RESP = handler::gen_obj(buf);
        resp_handler(id, event_id, res);
    })
}

pub fn send_req<REQ: handler::Event + serde::Serialize>(sender: &NioSender, req: &REQ) {
    let buffer = handler::gen_message(req);
    sender.send(buffer).unwrap();
}

struct IdManager {
    id: usize,
}

impl IdManager {
    fn new() -> IdManager {
        IdManager {
            id: 0
        }
    }

    fn get_next_id(&mut self) -> usize {
        self.id += 1;
        self.id
    }

}


