use std::{
    collections::HashMap,
    marker::PhantomPinned,
    pin::Pin,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
};

use twizzler_abi::syscall::{
    sys_thread_sync, ThreadSync, ThreadSyncFlags, ThreadSyncOp, ThreadSyncReference,
    ThreadSyncSleep, ThreadSyncWake,
};
use twizzler_runtime_api::ObjID;

use super::{ManagedThreadRef, THREAD_MGR};

pub(super) struct ThreadCleaner {
    thread: std::thread::JoinHandle<()>,
    send: Sender<WaitOp>,
    inner: Pin<Arc<ThreadCleanerData>>,
}

#[derive(Default)]
struct ThreadCleanerData {
    notify: AtomicU64,
    _unpin: PhantomPinned,
}

#[derive(Default)]
struct Waits {
    threads: HashMap<ObjID, ManagedThreadRef>,
}

// Changes to the collection of threads we are tracking
enum WaitOp {
    Add(ManagedThreadRef),
    Remove(ObjID),
}

impl ThreadCleaner {
    pub(super) fn new() -> Self {
        let (send, recv) = std::sync::mpsc::channel();
        let data = Arc::pin(ThreadCleanerData::default());
        let inner = data.clone();
        let thread = std::thread::Builder::new()
            .name("thread-exit cleanup tracker".into())
            .spawn(move || cleaner_thread_main(data, recv))
            .unwrap();
        Self {
            send,
            inner,
            thread,
        }
    }

    /// Track a thread. If that thread exits, the cleanup thread will remove the exited thread from
    /// tracking and from the global thread manager.
    pub fn track(&self, th: ManagedThreadRef) {
        tracing::debug!("tracking thread {}", th.id);
        let _ = self.send.send(WaitOp::Add(th));
        self.inner.notify();
    }

    /// Untrack a thread. Threads removed this way do not trigger a removal from the global thread
    /// manager.
    pub fn untrack(&self, id: ObjID) {
        let _ = self.send.send(WaitOp::Remove(id));
        self.inner.notify();
    }
}

impl ThreadCleanerData {
    /// Notify the cleanup thread that new items are on the queue.
    fn notify(&self) {
        self.notify.store(1, Ordering::SeqCst);
        let mut ops = [ThreadSync::new_wake(ThreadSyncWake::new(
            ThreadSyncReference::Virtual(&self.notify),
            1,
        ))];
        if let Err(e) = sys_thread_sync(&mut ops, None) {
            tracing::warn!("thread sync error when trying to notify: {}", e);
        }
    }
}

impl Waits {
    fn process_queue(&mut self, recv: &mut Receiver<WaitOp>) {
        while let Ok(wo) = recv.try_recv() {
            match wo {
                WaitOp::Add(th) => {
                    self.threads.insert(th.id, th);
                }
                WaitOp::Remove(id) => {
                    self.threads.remove(&id);
                }
            }
        }
    }
}

#[tracing::instrument(skip(data, recv))]
fn cleaner_thread_main(data: Pin<Arc<ThreadCleanerData>>, mut recv: Receiver<WaitOp>) {
    tracing::error!("CLEAN");
    // TODO (dbittman): when we have support for async thread events, we can use that API.
    let mut ops = Vec::new();
    let mut cleanups = Vec::new();
    let mut waits = Waits::default();
    loop {
        ops.truncate(0);
        // Apply any waiting operations.
        waits.process_queue(&mut recv);
        tracing::debug!("cleaner: waiting: {:?}", waits.threads.keys());

        // Add the notify sleep op.
        ops.push(ThreadSync::new_sleep(ThreadSyncSleep::new(
            ThreadSyncReference::Virtual(&data.notify),
            0,
            ThreadSyncOp::Equal,
            ThreadSyncFlags::empty(),
        )));

        // Add all sleep ops for threads.
        cleanups.extend(waits.threads.extract_if(|_, th| th.has_exited()));
        for th in waits.threads.values() {
            ops.push(ThreadSync::new_sleep(th.waitable_until_exit()));
        }

        // Remove any exited threads from the thread manager.
        for (_, th) in cleanups.drain(..) {
            tracing::debug!("cleaning thread: {}", th.id);
            THREAD_MGR.do_remove(&th);
        }

        // Check for notifications, and sleep.
        if data.notify.swap(0, Ordering::SeqCst) == 0 {
            // no notification, go to sleep. hold the lock over the sleep so that someone cannot
            // modify waits.threads on us while we're asleep.
            tracing::debug!("==> {:?}", ops);
            if let Err(e) = sys_thread_sync(&mut ops, None) {
                tracing::warn!("thread sync error: {}", e);
            }
        }
    }
}
