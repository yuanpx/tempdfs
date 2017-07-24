extern crate futures;
extern crate tokio_io;
extern crate tokio_core;
use std::collections::HashMap;
use std::vec::Vec;
use futures::future::Future;

use tokio_core::reactor::Handle;
use tokio_core::net;
use tokio_core::io;
use std::net::TcpStream;
use std::io::{Error, BufReader};
use tokio_io::AsyncRead;
use futures::future::IntoFuture;
use std::rc::Rc;
use std::cell::RefCell;
use std::time::{Duration, Instant};
use tokio_core::reactor::Timeout;
use std::ops::DerefMut;

use super::handler;

struct BizurConfig {
    addrs: Vec<String>,
    timeout: u64,
}


struct BizurSerive {
    elect_id: u64,
    voted_id: u64,
    vals: HashMap<String, String>,
    net_handle: Handle,
    remotes: Rc<HashMap<String, TcpStream>>,
    node_count: u32,
    is_leader: bool,
    leader: Option<String>,
    config: Option<BizurConfig>,
    id: String,
    voted_count: u32,
    heart_beat: bool,
}

#[derive(Serialize, Deserialize)]
struct VoteReq {
    elect_id: u64,
    addr: String,
}

impl handler::Event for VoteReq {
    fn event_id() -> u32 {
        4
    }
}

#[derive(Serialize, Deserialize)]
struct VoteResp {
    elect_id: u64,
    ack: u8, //1: ack, 2: nack
}

impl handler::Event for VoteResp {
    fn event_id() -> u32 {
        5
    }
}

#[derive(Serialize, Deserialize)]
struct HeartBeat {
    bit: u8,
}


fn get_major_count(service: &BizurSerive) -> u32 {
    return service.node_count/2 + 1;
}

fn start_election(service: &mut Rc<RefCell<BizurSerive>>) {
    service.borrow_mut().elect_id += 1;
    service.borrow_mut().voted_count = 0;
    service.borrow_mut().is_leader = false;
    let elect_id = service.borrow().elect_id;
    let config = service.borrow_mut().config.take().unwrap();

    for con_addr in &config.addrs {
        let mut remotes = service.borrow_mut().remotes.clone();
        let con = remotes.get(con_addr).unwrap();
        let source = service.borrow().id.clone();
        let please_resp = {
            //let con_inner = con.try_clone().unwrap();
            let con_inner = con.try_clone().unwrap();
            let async_con = net::TcpStream::from_stream(con_inner, &service.borrow_mut().net_handle).unwrap();
            let (reader, writer) = async_con.split();
            let vote_req = VoteReq{
                elect_id: elect_id,
                addr: source,
            };

            let req_message = handler::gen_message(&vote_req);
            let req = io::write_all(writer , req_message);
            let resp=  req.and_then(move|(_, _)| {
                let reader = BufReader::new(reader);
                let vote_resp = VoteResp {
                    elect_id: 0,
                    ack: 0,
                };

                let resp_message = handler::gen_message(&vote_resp);

                let resp =  io::read_exact(reader, resp_message);
                resp.and_then(move|(_, resp_message)| {
                    // Ok(body)
                    let vote_resp: VoteResp = handler::gen_obj(&resp_message);
                    


                    Ok(())
                })
            });
            resp.map_err(|_|())
        }; 

        service.borrow_mut().net_handle.spawn(please_resp);
    }
    
    service.borrow_mut().config = Some(config);
}


fn send_vote_resp(service: &mut BizurSerive, resp: &VoteResp) {
    
}

fn handle_vote_req(service: &mut BizurSerive, req: &mut VoteReq) {
    if req.elect_id > service.voted_id {
        service.voted_id = req.elect_id;
        service.leader = Some(req.addr.clone());
        let resp = VoteResp  {
            elect_id: req.elect_id,
            ack: 1, 
        };
        send_vote_resp(service, &resp);
    } else if req.elect_id == service.voted_id && service.leader.as_ref().unwrap() == &req.addr{
        let resp = VoteResp  {
            elect_id: req.elect_id,
            ack: 1, 
        };
        send_vote_resp(service, &resp);
    } else {
        let resp = VoteResp  {
            elect_id: req.elect_id,
            ack: 2, 
        };

        send_vote_resp(service, &resp);
    }
}

fn handle_vote_resp(service: &mut BizurSerive, resp: &mut VoteResp) {
    if resp.elect_id == service.elect_id {
        if resp.ack == 1 {
            service.voted_count += 1;
            if service.voted_count > get_major_count(service) {
                service.is_leader = true;
            }
        }
    }
}

fn send_endpoint_heartbeat(service: &mut BizurSerive, addr: &str) {
    
}

fn start_send_heart_beat(service: Rc<RefCell<BizurSerive>>) {
    if service.borrow().is_leader {
        let config = service.borrow_mut().config.take().unwrap();
        for endpoint in &config.addrs {
            send_endpoint_heartbeat(service.borrow_mut().deref_mut(), endpoint);
        }

        let dur = Duration::from_secs(config.timeout);
        let heart_beat_timeout = Timeout::new(dur, &service.borrow_mut().net_handle).unwrap();
        let service_inner = service.clone();
        let heart_beat_timeout = heart_beat_timeout.and_then(move |_| {
            start_send_heart_beat(service_inner);
            Ok(())
        });
        let heart_beat_timeout = heart_beat_timeout.map_err(|_|());
        service.borrow_mut().net_handle.spawn(heart_beat_timeout);

        service.borrow_mut().config = Some(config);
    }
}

fn start_check_heartbeat(service: &mut Rc<RefCell<BizurSerive>>) {
    if !service.borrow_mut().is_leader {
        if service.borrow_mut().heart_beat {
            let config = service.borrow_mut().config.take().unwrap();
            let dur = Duration::from_secs(config.timeout);
            let heart_beat_timeout = Timeout::new(dur, &service.borrow_mut().net_handle).unwrap();
            let mut service_inner = service.clone();
            let heart_beat_timeout = heart_beat_timeout.and_then(move |_| {
                start_check_heartbeat(&mut service_inner);
                Ok(())
            });
            let heart_beat_timeout = heart_beat_timeout.map_err(|_|());
            service.borrow_mut().net_handle.spawn(heart_beat_timeout);

            service.borrow_mut().config = Some(config);
        }
    } else {
        start_election(service);
    }
}








