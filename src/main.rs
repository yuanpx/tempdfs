#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate rmp;
extern crate rmp_serde as rmps;


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
use tokio_io::io;
use tokio_io::AsyncRead;
use std::sync::mpsc;
use std::net::SocketAddr;



use std::vec::Vec;

mod service;

type process_fn<T, I> = fn(&mut T, tx: service::nio_sender, I, &[u8]);



struct Context<T> {
    methods: Option<HashMap<u32, process_fn<T, SocketAddr>>>,
    connections: HashMap<SocketAddr, service::nio_sender>,
    service: Option<T>,
}


impl<T> Context<T> {
    fn new(t: T) -> Self {
        Context {
            methods: Some(HashMap::new()),
            connections: HashMap::new(),
            service: Some(t),
        }
    }
}

fn main() {
    println!("Hello, world!");

    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8180".to_string());
    let addr = addr.parse().unwrap();

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let socket = TcpListener::bind(&addr, &handle).unwrap();
    println!("Listening on: {}", addr);

    let (dtx, drx) = mpsc::channel();
    let mut service = service::Service::new(dtx.clone());

    // let connections = Rc::new(RefCell::new(HashMap::new()));
    let main_context = Rc::new(RefCell::new(Context::new(service)));
    let mut process_methods = main_context.borrow_mut().methods.take().unwrap();
    process_methods.insert(1, service::test_process_event);

    main_context.borrow_mut().methods = Some(process_methods);


    let srv = socket.incoming().for_each(move |(stream, addr)| {
        println!("New Connection: {}", addr);
        let (reader, writer) = stream.split();
        let (tx, rx) = futures::sync::mpsc::unbounded::<Vec<u8>>();
        // connections.borrow_mut().insert(addr, tx);
        main_context.borrow_mut().connections.insert(addr, tx);
        let main_context_inner = main_context.clone();
        let reader = BufReader::new(reader);

        let iter = stream::iter(iter::repeat(()).map(Ok::<(), Error>));
        let socket_reader = iter.fold((reader, main_context_inner),
                                      move |(reader, main_context_inner), _| {
            let header_buf: [u8; 4] = [0; 4];
            let header = io::read_exact(reader, header_buf);
            let body = header.and_then(|(reader, header)| {
                let body_len: u32 = service::handler::get_u32_length(&header[..]);
                let buff: Vec<u8> = vec![0;body_len as usize];
                io::read_exact(reader, buff)
            });

            body.map(move |(reader, vec)| {
                let event_id: u32 = service::handler::get_u32_length(&vec[0..4]);
                {
                    let process_methods = main_context_inner.borrow_mut().methods.take().unwrap();
                    let mut service = main_context_inner.borrow_mut().service.take().unwrap();
                    let mut tx = {
                        main_context_inner.borrow_mut().connections.get(&addr).unwrap().clone()
                    };

                    {
                        let method = process_methods.get(&event_id).unwrap();
                        method(&mut service, tx, addr, &vec[4..]);
                    }

                    main_context_inner.borrow_mut().methods = Some(process_methods);
                    main_context_inner.borrow_mut().service = Some(service);
                }
                (reader, main_context_inner)
            })
        });

        let socket_writer = rx.fold(writer, |writer, msg| {
            let amt = io::write_all(writer, msg);
            let amt = amt.map(|(writer, _)| writer);
            amt.map_err(|_| ())
        });

        // let connections_in = connections.clone();
        let main_context_in = main_context.clone();
        let socket_reader = socket_reader.map_err(|_| ());
        let connection = socket_reader.map(|_| ()).select(socket_writer.map(|_| ()));
        handle.spawn(connection.then(move |_| {
            // connections_in.borrow_mut().remove(&addr);
            main_context_in.borrow_mut().connections.remove(&addr);
            println!("Connection {} closed.", addr);
            Ok(())
        }));

        Ok(())
    });

    core.run(srv).unwrap();
}
