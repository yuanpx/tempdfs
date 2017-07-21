extern crate futures;
pub mod dio;
pub mod handler;
use std::net::SocketAddr;

use std::sync::mpsc;
pub type nio_sender = futures::sync::mpsc::UnboundedSender<Vec<u8>>;
pub type IdType = SocketAddr;

pub struct Service {
    dio_tx: mpsc::Sender<dio::DioCmd>,
}


pub fn test_process_event(_: &mut Service, _ : nio_sender, id: IdType, _: &[u8]) {
    println!("test: {}", id);
}


impl  Service {
    pub fn new(tx: mpsc::Sender<dio::DioCmd>) -> Self {
        Service{
            dio_tx: tx,
        }
    }
}

pub fn handle_read_file(service: &mut Service,tx: nio_sender, _: u32, buf: &[u8]) {
    let read_file_op: ReadFileOp = handler::gen_obj(buf);
    let mut read_op = dio::ReadOp {
        file_name: read_file_op.file_name,
        tx: tx,
    };

    service.dio_tx.send(dio::DioCmd::Read(read_op)).unwrap();
}


#[derive(Serialize, Deserialize)]
struct ReadFileOp {
    file_name: String,
}

impl handler::Event for ReadFileOp {
    fn event_id() -> u32 {
        2
    }
}

#[derive(Serialize, Deserialize)]
struct FileBuffer{
    buf_type: u8, //1:part, 2.end
    file_buf: Vec<u8>
}

impl handler::Event for FileBuffer {
    fn event_id() -> u32 {
        3
    }
}
