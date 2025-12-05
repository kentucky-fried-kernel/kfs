use crate::{
    buddy_allocator_levels, printkln,
    vmm::{
        allocators::backend::{
            buddy::{BUDDY_ALLOCATOR_SIZE, BuddyAllocator},
            slab::{SLAB_CACHE_SIZES, SlabAllocator},
        },
        paging::{
            Access, PAGE_SIZE, Permissions,
            mmap::{Mode, mmap},
        },
    },
};

use core::ptr::NonNull;

mod list;
mod state;

pub use list::{IntrusiveLink, List};

#[derive(Debug)]
pub enum KmallocError {
    NotEnoughMemory,
}

#[derive(Debug)]
pub enum KfreeError {
    InvalidPointer,
}

#[allow(unused)]
pub struct KernelAllocator {
    buddy_allocator: BuddyAllocator,
    slab_allocator: SlabAllocator,
}

static mut KERNEL_ALLOCATOR: KernelAllocator = KernelAllocator {
    buddy_allocator: unsafe { BuddyAllocator::new(None, BUDDY_ALLOCATOR_SIZE, buddy_allocator_levels!()) },
    slab_allocator: SlabAllocator::default(),
};

#[allow(static_mut_refs)]
pub fn kfree(addr: *const u8) -> Result<(), KfreeError> {
    unsafe { KERNEL_ALLOCATOR.buddy_allocator.free(addr) }
}

#[allow(static_mut_refs)]
pub fn kmalloc(size: usize) -> Result<*mut u8, KmallocError> {
    unsafe { KERNEL_ALLOCATOR.buddy_allocator.alloc(size).map_err(|_| KmallocError::NotEnoughMemory) }
}

/// Direct access to buddy allocator for testing purposes.
///
/// # Safety
/// This bypasses the slab allocator and should only be used in tests.
/// Normal code should use `kmalloc()` instead.
#[doc(hidden)]
#[allow(static_mut_refs)]
#[cfg(any(test, feature = "test-utils"))]
pub fn buddy_allocator_alloc(size: usize) -> Result<*mut u8, KmallocError> {
    unsafe { KERNEL_ALLOCATOR.buddy_allocator.alloc(size).map_err(|_| KmallocError::NotEnoughMemory) }
}

/// Direct access to buddy allocator for testing purposes.
///
/// # Safety
/// This bypasses the slab allocator and should only be used in tests.
/// Normal code should use `kmalloc()` instead.
#[doc(hidden)]
#[allow(static_mut_refs)]
#[cfg(any(test, feature = "test-utils"))]
pub fn buddy_allocator_free(addr: *const u8) -> Result<(), KfreeError> {
    unsafe { KERNEL_ALLOCATOR.buddy_allocator.free(addr) }
}

#[allow(static_mut_refs)]
pub fn init_buddy_allocator() -> Result<(), KmallocError> {
    let cache_memory = mmap(None, BUDDY_ALLOCATOR_SIZE, Permissions::ReadWrite, Access::Root, Mode::Continous).map_err(|_| KmallocError::NotEnoughMemory)?;

    let buddy_allocator = unsafe { &mut KERNEL_ALLOCATOR.buddy_allocator };
    buddy_allocator.set_root(NonNull::new(cache_memory as *mut u8).ok_or(KmallocError::NotEnoughMemory)?);
    Ok(())
}

#[allow(static_mut_refs)]
pub fn init_slab_allocator(buddy_allocator: &mut BuddyAllocator) -> Result<(), KmallocError> {
    let slab_allocator = unsafe { &mut KERNEL_ALLOCATOR.slab_allocator };

    for (idx, size) in SLAB_CACHE_SIZES.iter().enumerate() {
        let slab_allocator_addr = buddy_allocator.alloc(PAGE_SIZE * 64).map_err(|_| KmallocError::NotEnoughMemory)?;

        let slab_allocator_addr = NonNull::new(slab_allocator_addr).ok_or(KmallocError::NotEnoughMemory)?;
        unsafe { slab_allocator.init_slab_cache(slab_allocator_addr, *size as usize, 64) }?;

        printkln!("{:?}", slab_allocator.caches()[idx]);
    }

    Ok(())
}

#[allow(static_mut_refs)]
pub fn init() -> Result<(), KmallocError> {
    init_buddy_allocator()?;
    init_slab_allocator(unsafe { &mut KERNEL_ALLOCATOR.buddy_allocator })?;

    Ok(())
}
