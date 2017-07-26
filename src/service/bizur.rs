extern crate futures;
extern crate tokio_io;
extern crate tokio_core;
extern crate toml;
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
use std::net::SocketAddr;
use super::handler::Event;
use super::bizur_conf;
use std::thread;
use super::http;


use std::sync::mpsc::Sender;

use super::handler;
#[derive(Serialize, Deserialize)]
struct VoteReq {
    elect_id: u64,
    addr: String,
}

impl VoteReq {
    fn clone(&self) -> VoteReq {
        VoteReq {
            elect_id: self.elect_id,
            addr: self.addr.clone(),
        }
    }
}

impl handler::Event for VoteReq {
    fn event_id() -> u32 {
        4
    }
}

pub enum BizurCmd {
    StartChecker,
    HttpReqLeader(Sender<String>),
}

enum LeaderStatus {
    Leader,
    NoHeartbeat,
    HeartBeat,
}


pub struct BizurService {
    elect_id: u64,
    voted_id: u64,
    vals: HashMap<String, String>,
    event_handle: Handle,
    remotes: Rc<HashMap<String, TcpStream>>,
    in_connections: HashMap<SocketAddr, super::NioSender>,
    node_count: u32,
    status: LeaderStatus,
    leader: String,
    config: bizur_conf::BizurConfig,
    voted_count: u32,
    heart_beat: bool,
    cmd_sender: futures::sync::mpsc::UnboundedSender<BizurCmd>,
}

impl super::FrameWork for BizurService {
    type LoopCmd = BizurCmd;

    fn new(path: &str, loop_cmd_sender: futures::sync::mpsc::UnboundedSender<Self::LoopCmd>, loop_handle: Handle) -> Self {
        let content = bizur_conf::load_config("/home/yuanpeixuan/workspace/tempdfs/bizur.toml").unwrap();
        let config: bizur_conf::BizurConfig = toml::from_str(&content).unwrap();

        let http_cmd_sender = loop_cmd_sender.clone();
        thread::spawn(|| {
            http::start_dashboard(http_cmd_sender);
        });

        BizurService {
            elect_id: 0,
            voted_id: 0,
            vals: HashMap::new(),
            event_handle: loop_handle,
            remotes: Rc::new(HashMap::new()),
            in_connections: HashMap::new(),
            node_count: 3,
            status: LeaderStatus::NoHeartbeat,
            leader: "".to_string(),
            config: config,
            voted_count: 0,
            heart_beat: false,
            cmd_sender: loop_cmd_sender,
        }
    }

    fn main_listen_addr(&self) -> &str {
        return &self.config.listen_addr;
    }

    fn handle_connect(service: Rc<RefCell<Self>> ,addr: &SocketAddr,nio_sender: super::NioSender) {
        service.borrow_mut().in_connections.insert(addr.clone(), nio_sender);
    }

    fn handle_close(service: Rc<RefCell<Self>>, addr: &SocketAddr) {
        service.borrow_mut().in_connections.remove(addr);
    }

    fn handle_con_event(service: Rc<RefCell<Self>>, addr: &SocketAddr, event_id: super::IdType, buf: &[u8]) {
        if event_id == VoteReq::event_id() {
            let mut vote_req: VoteReq = super::handler::gen_obj(buf);
            handle_vote_req(service.borrow_mut().deref_mut(), &mut vote_req);
        }
        
    }

    fn handle_loop_event(service: Rc<RefCell<Self>>, cmd: Self::LoopCmd){
        match cmd {
            BizurCmd::HttpReqLeader(sender) => {
                let leader_string = "leader is ".to_string() + &service.borrow().leader;
                sender.send(leader_string);
            },
            _ => {},
        };
        
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


fn get_major_count(service: &BizurService) -> u32 {
    return service.node_count/2 + 1;
}

fn start_election(service: &Rc<RefCell<BizurService>>) {
    service.borrow_mut().elect_id += 1;
    service.borrow_mut().voted_count = 0;
    service.borrow_mut().status = LeaderStatus::NoHeartbeat;
    let elect_id = service.borrow().elect_id;
    let config = &service.borrow_mut().config;

    for con_addr in &config.addrs {
        if con_addr == &service.borrow().config.host {
            service.borrow_mut().voted_count = 1;
            continue;
        }

        let remotes = service.borrow_mut().remotes.clone();
        let con = remotes.get(con_addr).unwrap();
        let source = config.host.clone();
        let service_inner = service.clone();
        let vote_req = VoteReq{
            elect_id: elect_id,
            addr: source,
        };
        let please_resp = {
            //let con_inner = con.try_clone().unwrap();
            let con_inner = con.try_clone().unwrap();
            let async_con = net::TcpStream::from_stream(con_inner, &service.borrow_mut().event_handle).unwrap();
            let (reader, writer) = async_con.split();
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
                    let mut vote_resp: VoteResp = handler::gen_obj(&resp_message);
                    handle_vote_resp(service_inner.borrow_mut().deref_mut(), &mut vote_resp);
                    Ok(())
                })
            });
            resp.map_err(|_|())
        }; 

        let dur = Duration::from_secs(config.req_timeout);
        let req_timeout = Timeout::new(dur, &service.borrow_mut().event_handle).unwrap();
        let service_inner_timeout = service.clone();
        let req_timeout = req_timeout.and_then(move |_| {
            handle_req_vote_timeout(service_inner_timeout.borrow_mut().deref_mut(), &vote_req);
            Ok(())
        });

        let req_timeout = req_timeout.map_err(|_| ());

        let req_vote = please_resp.select(req_timeout).map(|_| ()).map_err(|_| ());

        service.borrow_mut().event_handle.spawn(req_vote);
    }
    
}


fn send_vote_resp(service: &mut BizurService, resp: &VoteResp) {
    
}

fn handle_vote_req(service: &mut BizurService, req: &mut VoteReq) {
    if req.elect_id > service.voted_id {
        service.voted_id = req.elect_id;
        service.leader = req.addr.clone();
        let resp = VoteResp  {
            elect_id: req.elect_id,
            ack: 1, 
        };
        send_vote_resp(service, &resp);
    } else if req.elect_id == service.voted_id && service.leader == req.addr{
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

fn handle_vote_resp(service: &mut BizurService, resp: &mut VoteResp) {
    if resp.elect_id == service.elect_id {
        if resp.ack == 1 {
            service.voted_count += 1;
            if service.voted_count > get_major_count(service) {
                service.status = LeaderStatus::Leader;
            }
        }
    }
}

fn handle_req_vote_timeout(service: &mut BizurService, vote_req: &VoteReq) {
    
}

fn send_endpoint_heartbeat(service: &mut BizurService, addr: &str) {
    
}

fn start_send_heart_beat(service: Rc<RefCell<BizurService>>) {
    match service.borrow().status {
        LeaderStatus::Leader =>  {
            let config = &service.borrow_mut().config;
            for endpoint in &config.addrs {
                send_endpoint_heartbeat(service.borrow_mut().deref_mut(), endpoint);
            }

            let dur = Duration::from_secs(config.heartbeat_timeout);
            let heart_beat_timeout = Timeout::new(dur, &service.borrow_mut().event_handle).unwrap();
            let service_inner = service.clone();
            let heart_beat_timeout = heart_beat_timeout.and_then(move |_| {
                start_send_heart_beat(service_inner);
                Ok(())
            });
            let heart_beat_timeout = heart_beat_timeout.map_err(|_|());
            service.borrow_mut().event_handle.spawn(heart_beat_timeout);
        },
        _ => {
            println!("no need to send heartbeat");
        }
    }
}

fn start_check_heartbeat(service: &Rc<RefCell<BizurService>>) {

    match service.borrow().status {
        LeaderStatus::HeartBeat => {
            service.borrow_mut().status = LeaderStatus::NoHeartbeat;
        },
        LeaderStatus::NoHeartbeat => {
            start_election(&service);
        },
        _ => {
            println!("be the leader, no need to check heartbeat");
        }
    };
    let config = &service.borrow_mut().config;
    let dur = Duration::from_secs(config.heartbeat_timeout);
    let heart_beat_timeout = Timeout::new(dur, &service.borrow_mut().event_handle).unwrap();
    let mut service_inner = service.clone();
    let heart_beat_timeout = heart_beat_timeout.and_then(move |_| {
        start_check_heartbeat(&mut service_inner);
        Ok(())
    });
    let heart_beat_timeout = heart_beat_timeout.map_err(|_|());
    service.borrow_mut().event_handle.spawn(heart_beat_timeout);
}
