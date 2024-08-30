use std::{marker::PhantomData, mem::size_of};

use twizzler_runtime_api::{FotResolveError, MapFlags, ObjID};

use super::{InvPtrBuilder, ResolvedPtr};
use crate::{marker::InvariantValue, object::RawObject};

#[derive(twizzler_derive::Invariant, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
#[repr(C)]
pub struct GlobalPtr<T> {
    id: ObjID,
    offset: u64,
    _pd: PhantomData<*const T>,
}

impl<T> Clone for GlobalPtr<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            offset: self.offset,
            _pd: PhantomData,
        }
    }
}

impl<T> Copy for GlobalPtr<T> {}

unsafe impl<T> InvariantValue for GlobalPtr<T> {}

impl<T> GlobalPtr<T> {
    pub const fn new(id: ObjID, offset: u64) -> Self {
        Self {
            id,
            offset,
            _pd: PhantomData,
        }
    }

    pub fn from_va(ptr: *const T) -> Option<Self> {
        let runtime = twizzler_runtime_api::get_runtime();
        let (handle, offset) = runtime.ptr_to_handle(ptr as *const u8)?;
        Some(Self {
            id: handle.id,
            offset: offset as u64,
            _pd: PhantomData,
        })
    }

    pub const fn id(&self) -> ObjID {
        self.id
    }

    pub const fn offset(&self) -> u64 {
        self.offset
    }

    pub const fn cast<U>(&self) -> GlobalPtr<U> {
        GlobalPtr::new(self.id, self.offset)
    }

    pub unsafe fn resolve(&self) -> Result<ResolvedPtr<'_, T>, FotResolveError> {
        // TODO: shouldn't use WRITE here?
        let handle = twizzler_runtime_api::get_runtime()
            .map_object(self.id(), MapFlags::READ | MapFlags::WRITE)?;
        let ptr = handle
            .lea(self.offset() as usize, size_of::<T>())
            .ok_or(FotResolveError::InvalidArgument)?;
        Ok(unsafe { ResolvedPtr::new_with_handle(ptr as *const T, handle) })
    }
}

// Safety: These are the standard library rules for references (https://doc.rust-lang.org/std/primitive.reference.html).
unsafe impl<T: Sync> Sync for GlobalPtr<T> {}
unsafe impl<T: Sync> Send for GlobalPtr<T> {}

impl<T> From<GlobalPtr<T>> for InvPtrBuilder<T> {
    fn from(value: GlobalPtr<T>) -> Self {
        Self::from_global(value)
    }
}
