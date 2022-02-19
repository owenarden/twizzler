use std::sync::atomic::{AtomicU64, Ordering};

use twizzler::object::{ObjID, ObjectInitFlags, Protections};
use twizzler_abi::syscall::{
    BackingType, LifetimeType, ObjectCreate, ObjectCreateFlags, ThreadSync, ThreadSyncFlags,
    ThreadSyncOp, ThreadSyncReference, ThreadSyncSleep, ThreadSyncWake,
};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

struct Rendezvous {
    ready: AtomicU64,
    tx_buf: ObjID,
    rx_buf: ObjID,
    tx_queue: ObjID,
    rx_queue: ObjID,
}

#[derive(Debug)]
pub struct NmOpenObjects {
    tx_buf: ObjID,
    rx_buf: ObjID,
    tx_queue: ObjID,
    rx_queue: ObjID,
}

fn wait_while_eq(pt: &AtomicU64, val: u64) {
    while pt.load(Ordering::SeqCst) == val {
        let op = ThreadSync::new_sleep(ThreadSyncSleep::new(
            ThreadSyncReference::Virtual(pt as *const AtomicU64),
            val,
            ThreadSyncOp::Equal,
            ThreadSyncFlags::empty(),
        ));
        let _ = twizzler_abi::syscall::sys_thread_sync(&mut [op], None);
    }
}

fn write_wake(pt: &AtomicU64, val: u64) {
    pt.store(val, Ordering::SeqCst);
    let op = ThreadSync::new_wake(ThreadSyncWake::new(
        ThreadSyncReference::Virtual(pt as *const AtomicU64),
        usize::MAX,
    ));
    let _ = twizzler_abi::syscall::sys_thread_sync(&mut [op], None);
}

fn new_obj() -> ObjID {
    let create = ObjectCreate::new(
        BackingType::Normal,
        LifetimeType::Volatile,
        None,
        ObjectCreateFlags::empty(),
    );
    twizzler_abi::syscall::sys_object_create(create, &[], &[]).unwrap()
}

pub fn server_rendezvous(rid: ObjID) -> NmOpenObjects {
    let mut obj = twizzler::object::Object::<Rendezvous>::init_id(
        rid,
        Protections::READ | Protections::WRITE,
        ObjectInitFlags::empty(),
    )
    .unwrap();
    let mut rendezvous = obj.base_raw_mut();
    wait_while_eq(&rendezvous.ready, 0);
    let o = NmOpenObjects {
        tx_buf: new_obj(),
        rx_buf: new_obj(),
        tx_queue: new_obj(),
        rx_queue: new_obj(),
    };
    rendezvous.tx_buf = o.tx_buf;
    rendezvous.rx_buf = o.rx_buf;
    rendezvous.tx_queue = o.tx_queue;
    rendezvous.rx_queue = o.rx_queue;
    write_wake(&rendezvous.ready, 2);
    o
}

pub fn client_rendezvous(rid: ObjID) -> NmOpenObjects {
    let obj = twizzler::object::Object::<Rendezvous>::init_id(
        rid,
        Protections::READ | Protections::WRITE,
        ObjectInitFlags::empty(),
    )
    .unwrap();
    let rendezvous = obj.base_raw();
    write_wake(&rendezvous.ready, 1);
    wait_while_eq(&rendezvous.ready, 1);
    let o = NmOpenObjects {
        tx_buf: rendezvous.tx_buf,
        rx_buf: rendezvous.rx_buf,
        tx_queue: rendezvous.tx_queue,
        rx_queue: rendezvous.rx_queue,
    };
    write_wake(&rendezvous.ready, 0);
    o
}
