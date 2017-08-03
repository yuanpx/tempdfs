pub mod dio;
pub mod handler;
pub mod bizur;
pub mod bizur_conf;
pub mod http;
pub mod ring;
pub type NioSender = futures::sync::mpsc::UnboundedSender<Vec<u8>>;
pub type IdType = u32;
extern crate futures;
extern crate tokio_core;
extern crate tokio_io;


use std::collections::HashMap;
use std::rc::Rc;
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


pub trait FrameWork {
    type LoopCmd;
    fn new(path: &str, loop_cmd_sender: futures::sync::mpsc::UnboundedSender<Self::LoopCmd>, loop_handle: Handle) -> Self;

    fn handle_loop_event(service: Rc<RefCell<Self>>, cmd: Self::LoopCmd);
}

pub trait NetEvent {

    fn gen_next_id(&mut self) -> usize;

    fn handle_connect(service: Rc<RefCell<Self>> ,addr: &SocketAddr, id: usize, nio_sender: NioSender);

    fn handle_close(service: Rc<RefCell<Self>>, addr: &SocketAddr, id: usize);

    fn handle_con_event(service: Rc<RefCell<Self>>, addr: &SocketAddr, id: usize, event_id: IdType, buf: &[u8]);
}



pub fn start_framework<T: 'static + FrameWork>(path: &str) {
    
    let (loop_cmd_tx, loop_cmd_rx) = futures::sync::mpsc::unbounded::<T::LoopCmd>();
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let loop_handler = handle.clone();
    let service = T::new(path, loop_cmd_tx, loop_handler);
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
                                              T::handle_con_event(service_handle_event, &addr,next_id, event_id , &vec[4..]);
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

pub fn start__listen<T: 'static + NetEvent>(service: Rc<RefCell<T>>, handle: Handle, addr: SocketAddr) {
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

pub struct RpcConnection {
    id: usize,
    sender: NioSender, 
    pendding_call: usize, 
}

type BuffHandler<T> = fn(service: Rc<RefCell<T>>, addr: &SocketAddr, id: IdType, buf: &[u8]);
type RpcHandler<T, RESP> = fn(service: Rc<RefCell<T>>, id: IdType, resp: RESP);

