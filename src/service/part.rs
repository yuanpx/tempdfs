extern crate futures;

use std::sync::mpsc;

use super::transaction::Transaction;
use super::replication_log::PartLogManager;

struct Part {
    id: usize,
}


pub struct PartWorker{
    part_id: usize,
    part_log_manager: super::replication_log::PartLogManager
}


pub type PartIoNotifySender = futures::sync::oneshot::Sender<()>;

pub struct PartIoCmd(super::transaction::Transaction,PartIoNotifySender);

pub fn part_io_worker_thread(rx: mpsc::Receiver<PartIoCmd>,mut  part_worker: PartWorker) {
    let mut continue_flag = true;

    while continue_flag {
        let mut part_io_cmd = rx.recv().unwrap();
        let PartIoCmd(transaction, sender) = part_io_cmd;
        handle_transaction(&mut part_worker, transaction);
        sender.send(()).unwrap();
    }
}

fn handle_transaction(worker: &mut PartWorker, transaction: Transaction) {
    
}


