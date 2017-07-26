extern crate futures;
extern crate hyper;

use futures::future::FutureResult;

use self::hyper::header::ContentLength;
use self::hyper::header::ContentType;
use self::hyper::server::{Http, Service, Request, Response};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender, Receiver};

use super::bizur::BizurCmd;
use std::sync::mpsc::channel;

struct Messager {
    req_sender: futures::sync::mpsc::UnboundedSender<BizurCmd>,
    resp_sender: Sender<String>,
    resp_receiver: Receiver<String>,
}

impl Messager {
    fn new(req_sender: futures::sync::mpsc::UnboundedSender<BizurCmd>) -> Self {
        let (sender, receiver) = channel();
        Messager {
            req_sender: req_sender,
            resp_sender: sender,
            resp_receiver: receiver,
        }
    }
}

struct Dashboard {
    inner: Arc<Mutex<Messager>>,
}

impl Dashboard {
    fn new(inner: Arc<Mutex<Messager>>) -> Self {

        Dashboard {
            inner: inner,
        }
    }
}



impl Service for Dashboard {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = FutureResult<Response, hyper::Error>;

    fn call(&self, req: Request) -> Self::Future {
        let msg = {
            let messager = self.inner.lock().unwrap();
            let sender = messager.resp_sender.clone();
            let req = BizurCmd::HttpReqLeader(sender);
            messager.req_sender.send(req).unwrap();
            let msg = messager.resp_receiver.recv().unwrap();
            msg
        };

        futures::future::ok(
            Response::new()
                .with_header(ContentLength(msg.len() as u64))
                .with_header(ContentType::plaintext())
                .with_body(msg))
    }
}


pub fn start_dashboard(req_sender: futures::sync::mpsc::UnboundedSender<BizurCmd>) {
    let addr = "127.0.0.1:3000".parse().unwrap();
    let messager = Arc::new(Mutex::new(Messager::new(req_sender)));
    let server = Http::new().bind(&addr, move||
                                  {
                                      let messager_inner = messager.clone();
                                      Ok(Dashboard::new(messager_inner))
                                  }

    ).unwrap();

    println!("http listen on 127.0.0.1:3000");
    server.run().unwrap();
    println!("http end!");
}
