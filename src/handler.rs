extern crate crypto;

use self::crypto::md5::Md5;
use self::crypto::digest::Digest;


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
    stage: u8,
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
