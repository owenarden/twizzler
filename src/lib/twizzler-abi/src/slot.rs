//! Manage slots in the address space. Currently not finished.

use core::sync::atomic::{AtomicUsize, Ordering};

/*
KANI_TODO
*/

use crate::{
    alloc::TwzGlobalAlloc,
    object::Protections,
    syscall::{MapFlags, ObjectCreate, ObjectCreateFlags},
};

pub(crate) struct Context {
    next_slot: AtomicUsize,
    pub alloc_lock: crate::simple_mutex::Mutex,
    pub global_alloc: TwzGlobalAlloc,
}

#[allow(dead_code)]
pub const RESERVED_TEXT: usize = 0;
#[allow(dead_code)]
pub const RESERVED_DATA: usize = 1;
#[allow(dead_code)]
pub const RESERVED_STACK: usize = 2;
#[allow(dead_code)]
pub const RESERVED_KERNEL_INIT: usize = 3;
#[allow(dead_code)]
pub(crate) const ALLOC_START: usize = 10;
