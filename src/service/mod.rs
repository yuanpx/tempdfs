pub mod dio;
pub mod handler;
pub mod bizur;
pub mod bizur_conf;
pub mod http;
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
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;
use tokio_core::reactor::Handle;
use tokio_io::io;
use tokio_io::AsyncRead;
use std::net::SocketAddr;


pub trait FrameWork {
    type LoopCmd;
    //type LoopCmdSender = futures::sync::mpsc::UnboundedSender<Self::LoopCmd>;
    //type LoopCmdReceiver = futures::sync::mpsc::UnboundedReceiver<Self::LoopCmd> ;
//    type LoopCmdSender;
//    type LoopCmdReceiver;

    //fn new(loop_cmd_sender: Self::LoopCmdSender) -> Self;
    fn new(loop_cmd_sender: futures::sync::mpsc::UnboundedSender<Self::LoopCmd>, loop_handle: Handle) -> Self;

    fn main_listen_addr(&self) -> &str;

    fn handle_connect(service: Rc<RefCell<Self>> ,addr: &SocketAddr,nio_sender: NioSender);

    fn handle_close(service: Rc<RefCell<Self>>, addr: &SocketAddr);

    fn handle_con_event(service: Rc<RefCell<Self>>, addr: &SocketAddr, id: IdType, buf: &[u8]);

    fn handle_loop_event(service: Rc<RefCell<Self>>, cmd: Self::LoopCmd);
}

pub fn start_framework<T: 'static + FrameWork>() {
    
    let (loop_cmd_tx, loop_cmd_rx) = futures::sync::mpsc::unbounded::<T::LoopCmd>();
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let loop_cmd_handle = handle.clone();
    let loop_handler = handle.clone();
    let service = T::new(loop_cmd_tx, loop_handler);
    let service = Rc::new(RefCell::new(service));
    let service_loop_handler = service.clone();
    let addr = service.borrow().main_listen_addr().to_string();
    let addr = addr.parse().unwrap();

    let socket = TcpListener::bind(&addr, &handle).unwrap();
    println!("Listening on: {}", addr);

    // let connections = Rc::new(RefCell::new(HashMap::new()));
    let srv = socket.incoming().for_each(move |(stream, addr)| {
        println!("New Connection: {}", addr);
        let (reader, writer) = stream.split();
        let (tx, rx) = futures::sync::mpsc::unbounded::<Vec<u8>>();
        // connections.borrow_mut().insert(addr, tx);
        T::handle_connect(service.clone(), &addr, tx);
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
                                              T::handle_con_event(service_handle_event, &addr, event_id , &vec[4..]);
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
            T::handle_close(service_in, &addr);
            println!("Connection {} closed.", addr);
            Ok(())
        }));

        Ok(())
    });

    let service_cmd_handler = service_loop_handler.clone();
    let loop_cmd_handler = loop_cmd_rx.fold(service_cmd_handler, |service_cmd_handler, cmd| {
        T::handle_loop_event(service_cmd_handler.clone(), cmd);
        Ok(service_cmd_handler)
    });

    let loop_cmd_handler = loop_cmd_handler.map(|_| ());
    let loop_cmd_handler = loop_cmd_handler.map_err(|_| ());

    loop_cmd_handle.spawn(loop_cmd_handler);
    core.run(srv).unwrap();
}




