extern crate crypto;
extern crate serde;
extern crate rmp;
extern crate rmp_serde as rmps;
use serde::Serialize;
use serde::Deserialize;
use rmps::Serializer;
use std::io::Cursor;
use rmps::Deserializer;

use std;

use self::crypto::md5::Md5;
use self::crypto::digest::Digest;
use std::fs::File;
use std::io::prelude::*;
use std::sync::mpsc;

mod dio;


pub fn get_u32_buff(len: u32) -> [u8;4] {
    unsafe {
        std::mem::transmute::<u32, [u8;4]>(len)
    }
}

pub fn get_u32_length(buf_slice: &[u8]) -> u32 {
    let mut buf: [u8;4] = [0;4];
    buf.copy_from_slice(buf_slice);
    unsafe {
        std::mem::transmute::<[u8;4], u32>(buf)
    }
    
}

pub trait Event {
    fn event_id() -> u32;
}

#[derive(Serialize, Deserialize)]
struct TestStruct {
    test_i8: i8,
    test_i32: i32,
}

#[derive(Serialize, Deserialize)]
struct ReadFileOp {
    file_name: String,
}

impl Event for ReadFileOp {
    fn event_id() -> u32 {
        2
    }
}

#[derive(Serialize, Deserialize)]
struct FileBuffer{
    buf_type: u8, //1:part, 2.end
    file_buf: Vec<u8>
}

impl Event for FileBuffer {
    fn event_id() -> u32 {
        3
    }
}


impl Event for TestStruct {
    fn event_id() -> u32 {
        1
    }
}

pub fn gen_message<T>(t: &T) -> Vec<u8>
    where T: Serialize + Event
{
    let mut obj_buf = Vec::new();
    t.serialize(&mut Serializer::new(&mut obj_buf)).unwrap();
    let mut buf = Vec::new();
    let mut obj_len: u32 = obj_buf.len() as u32;
    obj_len = obj_len + 4;
    let event_id: u32 = T::event_id();
    let obj_len_buf = get_u32_buff(obj_len);
    let event_id_buf = get_u32_buff(event_id);
    buf.extend_from_slice(&obj_len_buf[..]);
    buf.extend_from_slice(&event_id_buf[..]);
    buf.append(&mut obj_buf);

    buf
}

pub fn gen_obj<'a, T>(buf: &[u8]) -> T
    where T: Deserialize<'a> {

    let cur = Cursor::new(buf);
    let mut de = Deserializer::new(cur);
    let obj: T = Deserialize::deserialize(&mut de).unwrap();
    obj
}



enum NioStorage {
    INIT,
    RECV,
    SEND,
    CLOSE,
}


pub struct TaskInfo {
    arg: TaskArg,
    data: Vec<u8>,
    offset: i32,
    req_count: i64,
    finish_callback: fn(),
    thread_data: fn(),
}

pub enum TaskArg {
    StorageClientInfo(),
}


enum ExtraInfo {
    StorageUploadInfo(),
    StorageSetMetaInfo(),
}

struct TrunkFullInfo {
    
}

struct StorageUploadInfo {
    if_gen_filename: bool,
    file_type: u8,
    if_sub_path_alloced: bool,
    master_filename: String,
    file_ext_name: String,
    formatted_ext_name: String,
    prefix_name: String,
    group_name: String,
    start_time: i32,
    trunk_info: TrunkFullInfo,
    before_open_callback: fn(),
    before_close_callback: fn(),
}

struct StorageSetMetaInfo {
    op_flag: u8,
    meta_buf: Vec<u8>,
}

pub struct StorageFileContext {
    filename: String,
    fname2log: String,
    op: u8,
    sync_flag: u8,
    calc_crc32: bool,
    open_flags: i32,
    file_hash_codes: [i32;4],
    crc32: i32,
    md5_context: Md5,
    extra_info: ExtraInfo,
    dio_thread_index: i32,
    timestamp2log: i32,
    delete_flag: i32,
    create_flag: i32,
    buff_offset: i32,
    fd: i32,
    start: u64,
    end: u64,
    offset: u64,

    done_callback: fn(),
    log_callback: fn(),

    tv_deal_start: i32,
}

struct StorageServer {
    
}

enum ExtraArg{}

pub struct StorageclientInfo {
    nio_thread_index: i32,
    canceled: i32,
    stage: NioStorage,
    storage_server_id: String,
    file_context: StorageFileContext,
    total_length: i64,
    total_offset: i64,
    request_length: i64,
    src_storage: StorageServer,
    deal_func: fn(),
    extra_arg: ExtraArg,
    clean_func: fn(),
}

pub struct Service {
    dio_tx: mpsc::Sender<>

}

pub fn test_process_event(_: &mut Service, event_id: u32, _: &[u8]) {
    println!("test: {}", event_id);
}




impl Gen for Service {
    fn new() -> Self {
        Service{}
    }
}

pub trait Gen {
    fn new() -> Self;
}

pub fn handle_read_file(_: &mut Service, _: u32, buf: &[u8]) {
    let read_file_op: ReadFileOp = gen_obj(buf);

}


