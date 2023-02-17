use core::alloc::Layout;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicUsize, Ordering};

use super::VirtAddr;
use alloc::{collections::BTreeMap, sync::Arc};
use twizzler_abi::object::ObjID;
use twizzler_abi::{device::CacheType, object::Protections};

use crate::{
    idcounter::{Id, IdCounter},
    mutex::{LockGuard, Mutex},
    obj::{pages::PageRef, ObjectRef},
};

#[derive(Ord, PartialOrd, PartialEq, Eq)]
pub struct Mapping {
    pub obj: ObjectRef,
    pub perms: MappingPerms,
    pub vmc: MemoryContextRef,
    pub slot: usize,
}

pub type MappingRef = Arc<Mapping>;

impl Mapping {
    pub fn new(obj: ObjectRef, vmc: MemoryContextRef, slot: usize, perms: MappingPerms) -> Self {
        Self {
            obj,
            vmc,
            slot,
            perms,
        }
    }
}

pub struct MemoryContextInner {
    slots: BTreeMap<usize, MappingRef>,
    thread_count: u64,
}

impl Default for MemoryContextInner {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MemoryContext {
    inner: Mutex<MemoryContextInner>,
    id: Id<'static>,
    upcall: AtomicUsize,
}

impl Default for MemoryContext {
    fn default() -> Self {
        Self::new()
    }
}

pub type MemoryContextRef = Arc<MemoryContext>;

impl PartialEq for MemoryContext {
    fn eq(&self, other: &Self) -> bool {
        let ida = { self.id.value() };
        let idb = { other.id.value() };
        ida == idb
    }
}

impl Eq for MemoryContext {}

impl PartialOrd for MemoryContext {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        let ida = { self.id.value() };
        let idb = { other.id.value() };
        ida.partial_cmp(&idb)
    }
}

impl Ord for MemoryContext {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        let ida = { self.id.value() };
        let idb = { other.id.value() };
        ida.cmp(&idb)
    }
}

bitflags::bitflags! {
    pub struct MappingPerms : u32 {
        const READ = 1;
        const WRITE = 2;
        const EXECUTE = 4;
    }
}

impl From<Protections> for MappingPerms {
    fn from(p: Protections) -> Self {
        let mut s = MappingPerms::empty();
        if p.contains(Protections::READ) {
            s.insert(MappingPerms::READ)
        }
        if p.contains(Protections::WRITE) {
            s.insert(MappingPerms::WRITE)
        }
        if p.contains(Protections::EXEC) {
            s.insert(MappingPerms::EXECUTE)
        }
        s
    }
}

bitflags::bitflags! {
    pub struct MapFlags: u64 {
        const READ= 0x1;
        const WRITE= 0x2;
        const EXECUTE= 0x4;
        const USER= 0x8;
        const GLOBAL= 0x10;
        const WIRED = 0x20;
    }
}

impl From<MappingPerms> for MapFlags {
    fn from(mp: MappingPerms) -> Self {
        let mut flags = MapFlags::empty();
        if mp.contains(MappingPerms::READ) {
            flags.insert(MapFlags::READ);
        }
        if mp.contains(MappingPerms::WRITE) {
            flags.insert(MapFlags::WRITE);
        }
        if mp.contains(MappingPerms::EXECUTE) {
            flags.insert(MapFlags::EXECUTE);
        }
        flags
    }
}

pub fn addr_to_slot(addr: VirtAddr) -> usize {
    (addr.raw() / (1 << 30)) as usize //TODO: arch-dep
}

static ID_COUNTER: IdCounter = IdCounter::new();
impl MemoryContextInner {
    pub fn new_blank() -> Self {
        todo!()
    }

    pub fn new() -> Self {
        todo!()
    }

    pub fn current() -> Self {
        todo!()
    }

    fn clear_mappings(&mut self) {
        self.slots.clear();
    }

    pub fn add_thread(&mut self) {
        self.thread_count += 1;
    }

    pub fn remove_thread(&mut self) {
        self.thread_count -= 1;
        if self.thread_count == 0 {
            self.clear_mappings();
        }
    }

    //   pub fn mappings_iter(&self, start: VirtAddr) -> MappingIter {
    //     MappingIter::new(self, start)
    //   }

    pub fn lookup_object(&self, addr: VirtAddr) -> Option<MappingRef> {
        self.slots.get(&addr_to_slot(addr)).map(Clone::clone)
    }

    pub fn map_object_page(&mut self, addr: VirtAddr, page: PageRef, perms: MappingPerms) {
        todo!()
    }

    pub fn insert_mapping(&mut self, mapping: MappingRef) {
        //TODO: return value
        self.slots.insert(mapping.slot, mapping);
    }

    pub fn clone_region(&mut self, other_ctx: &MemoryContextInner, addr: VirtAddr) {
        todo!()
    }

    pub fn switch(&self) {
        todo!()
    }
}

impl MemoryContext {
    pub fn new_blank() -> Self {
        todo!()
    }

    pub fn new() -> Self {
        todo!()
    }

    pub fn current() -> Self {
        todo!()
    }

    pub fn switch(&self) {
        todo!()
    }

    pub fn inner(&self) -> LockGuard<'_, MemoryContextInner> {
        self.inner.lock()
    }

    pub fn set_upcall_address(&self, target: usize) {
        self.upcall.store(target, Ordering::SeqCst);
    }

    pub fn get_upcall_address(&self) -> Option<usize> {
        match self.upcall.load(Ordering::SeqCst) {
            0 => None,
            n => Some(n),
        }
    }
}

use crate::syscall::object::ObjectHandle;
impl ObjectHandle for MemoryContextRef {
    fn create_with_handle(_obj: ObjectRef) -> Self {
        Arc::new(MemoryContext::new())
    }
}

pub mod virtmem;

/// A trait that defines the operations expected by higher-level object management routines. An architecture-dependent
/// type can be created that implements Context, which can then be used by the rest of the kernel to manage objects in a
/// context (e.g. an address space).
pub trait Context {
    /// The type that is expected for upcall information (e.g. an entry address).
    type UpcallInfo;
    /// The type that is expected for informing the context how to map the object (e.g. a slot number).
    type MappingInfo;

    /// Set the context's upcall information.
    fn set_upcall(&self, target: Self::UpcallInfo);
    /// Retrieve the context's upcall information.
    fn get_upcall(&self) -> Option<Self::UpcallInfo>;
    /// Switch to this context.
    fn switch_to(&self);
    /// Insert a range of an object into the context. The implementation may choose to use start and len as hints, but
    /// should keep in mind that calls to `insert_object` may be generated by faults, and so should strive to resolve
    /// the fault by correctly mapping the object as requested.
    fn insert_object(
        &self,
        obj: ObjectRef,
        mapping_info: Self::MappingInfo,
        perms: MappingPerms,
        cache: crate::memory::map::CacheType,
    ) -> Result<(), InsertError>;
    /// Remove an object's mapping from the context.
    fn remove_object(&self, obj: ObjID, start: usize, len: usize);
    /// Write protect a region of the object's mapping. For correctness, the implementation must ensure that the region
    /// is, indeed, write protected. If this means protecting the entire object, so be it.
    fn write_protect(&self, obj: ObjID, start: usize, len: usize);
}

/// Errors for inserting objects into a [Context].
pub enum InsertError {
    Occupied,
}

/// A trait for kernel-related memory context actions.
pub(super) trait KernelMemoryContext {
    /// Allocate a contiguous chunk of memory. This is not expected to be good for small allocations, this should be
    /// used to grab large chunks of memory to then serve pieces of using an actual allocator. Returns a pointer to the
    /// allocated memory and the size of the allocation (must be greater than layout's size).
    fn allocate_chunk(&self, layout: Layout) -> NonNull<u8>;
    /// Deallocate a previously allocated chunk.
    ///
    /// # Safety
    /// The call must ensure that the passed in pointer came from a call to [Self::allocate_chunk] and has the same
    /// layout data as was passed to that allocation call.
    unsafe fn deallocate_chunk(&self, layout: Layout, ptr: NonNull<u8>);
}

lazy_static::lazy_static! {
    // TODO: make these more like impl Context (or conditional compilation).
    static ref KERNEL_CONTEXT: virtmem::VirtContext = {
        let c = virtmem::VirtContext::new_kernel();
        c.init_kernel_context();
        c
    };
}

pub fn kernel_context() -> &'static virtmem::VirtContext {
    &KERNEL_CONTEXT
}
