use crate::{
    buddy_allocator_levels,
    vmm::{
        allocators::backend::{
            buddy::{BUDDY_ALLOCATOR_SIZE, BuddyAllocator},
            slab::{SLAB_CONFIGS, SlabAllocator},
        },
        paging::{
            Access, PAGE_SIZE, Permissions,
            mmap::{Mode, mmap},
        },
    },
};

use core::{alloc::GlobalAlloc, ptr::NonNull};

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
    pub buddy_allocator: BuddyAllocator,
    pub slab_allocator: SlabAllocator,
}

/// # Safety:
/// If any of the following conditions are violated, the result is Undefined Behavior:
/// * The following initializations must have been made before allocating anything through the
///   [`kfs::alloc`] API:
///   * The paging-related data structures and registers must be ready to be used
///     ([`kfs::vmm::paging::init::init_memory`])
///   * The data structures used for dynamic memory allocation (buddy and slab allocators) must have
///     been initialized ([`kfs::vmm::allocators::kmalloc::init`])
///   * _Ideally_, the IDT should also be initialized ([`kfs::arch::x86::idt::init`]) in order to
///     catch possible page faults
unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        kmalloc(layout.size().max(layout.align())).unwrap_or_default()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        // SAFETY:
        // Passing a random pointer to `kfree` would result in undefined behavior, but since we rely on
        // rustc to insert all allocation/free operations, we can safely assume that no invalid
        // pointers will be passed to this function.
        assert!(unsafe { kfree(ptr) }.is_ok(), "Freed invalid pointer");
    }
}

#[cfg(all(not(test), not(feature = "test-utils")))]
#[global_allocator]
#[allow(clippy::multiple_unsafe_ops_per_block)]
static mut KERNEL_ALLOCATOR: KernelAllocator = KernelAllocator {
    // SAFETY:
    // - We are creating references to static mutable variables. We make this safe by ensuring that the buddy allocator is the sole owner of these references,
    //   and they are never touched by anything without going through the buddy allocator's API.
    // - The safety requirements regarding the `root` argument of `BuddyAllocator::new()` do not apply, since we are initializing it with `None.
    buddy_allocator: { unsafe { BuddyAllocator::new(None, BUDDY_ALLOCATOR_SIZE, buddy_allocator_levels!()) } },
    slab_allocator: SlabAllocator::default(),
};

#[cfg(any(test, feature = "test-utils"))]
#[global_allocator]
#[allow(clippy::multiple_unsafe_ops_per_block)]
pub static mut KERNEL_ALLOCATOR: KernelAllocator = KernelAllocator {
    // SAFETY:
    // - We are creating references to static mutable variables. We make this safe by ensuring that the buddy allocator is the sole owner of these references,
    //   and they are never touched by anything without going through the buddy allocator's API.
    // - The safety requirements regarding the `root` argument of `BuddyAllocator::new()` do not apply, since we are initializing it with `None.
    buddy_allocator: { unsafe { BuddyAllocator::new(None, BUDDY_ALLOCATOR_SIZE, buddy_allocator_levels!()) } },
    slab_allocator: SlabAllocator::default(),
};

/// # Safety
/// This function will interact with the kernel allocator, and therefore
/// dereference raw pointers and all other sorts of bad stuff. It is the
/// caller's responsibility to only _ever_ call this if the kernel allocator is
/// properly initialized.
///
/// # Errors
/// This function will return an error if `addr` is not pointing to an allocated
/// block of memory.
#[allow(static_mut_refs)]
pub unsafe fn kfree(addr: *const u8) -> Result<(), KfreeError> {
    // SAFETY:
    // We are accessing a static mutable allocator, which is only accessible through this crate. The API
    // of this crate ensures we are not touching it outside of its expected usage.
    match unsafe { KERNEL_ALLOCATOR.slab_allocator.free(addr) } {
        Ok(()) => Ok(()),
        // SAFETY:
        // We are accessing a static mutable allocator, which is only accessible through this crate. The API
        // of this crate ensures we are not touching it outside of its expected usage.
        Err(_) => match unsafe { KERNEL_ALLOCATOR.buddy_allocator.free(addr) } {
            Ok(()) => Ok(()),
            Err(_) => Err(KfreeError::InvalidPointer),
        },
    }
}

/// # Errors
/// This function will return an error if it fails to find a sufficiently large
/// block of memory for the allocation.
#[allow(static_mut_refs)]
pub fn kmalloc(size: usize) -> Result<*mut u8, KmallocError> {
    match size {
        // SAFETY:
        // We are accessing a static mutable allocator, which is only accessible through this crate. The API
        // of this crate ensures we are not touching it outside of its expected usage.
        0..=2048 => unsafe { KERNEL_ALLOCATOR.slab_allocator.alloc(size).map_err(|_| KmallocError::NotEnoughMemory) },
        // SAFETY:
        // We are accessing a static mutable allocator, which is only accessible through this crate. The API
        // of this crate ensures we are not touching it outside of its expected usage.
        2049.. => unsafe {
            KERNEL_ALLOCATOR
                .buddy_allocator
                .alloc(1 << ((size - 1).ilog2() + 1))
                .map_err(|_| KmallocError::NotEnoughMemory)
        },
    }
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
    // SAFETY:
    // We are accessing a static mutable allocator, which is only accessible through this crate. The API
    // of this crate ensures we are not touching it outside of its expected usage.
    unsafe { KERNEL_ALLOCATOR.buddy_allocator.alloc(size).map_err(|_| KmallocError::NotEnoughMemory) }
}

/// Direct access to buddy allocator for testing purposes.
///
/// # Safety
/// This bypasses the slab allocator and should only be used in tests.
/// Normal code should use `kfree()` instead.
#[doc(hidden)]
#[allow(static_mut_refs)]
#[cfg(any(test, feature = "test-utils"))]
pub fn buddy_allocator_free(addr: *const u8) -> Result<(), KfreeError> {
    // SAFETY:
    // We are accessing a static mutable allocator, which is only accessible through this crate. The API
    // of this crate ensures we are not touching it outside of its expected usage.
    unsafe { KERNEL_ALLOCATOR.buddy_allocator.free(addr) }
}

/// # Errors
/// This function will return an error if the initial allocation for the
/// `BuddyAllocator` (made via `mmap`) fails.
#[allow(static_mut_refs)]
pub fn init_buddy_allocator(buddy_allocator: &mut BuddyAllocator) -> Result<(), KmallocError> {
    let cache_memory = mmap(None, BUDDY_ALLOCATOR_SIZE, Permissions::ReadWrite, Access::Root, &Mode::Continous).map_err(|_| KmallocError::NotEnoughMemory)?;

    buddy_allocator.set_root(NonNull::new(cache_memory as *mut u8).ok_or(KmallocError::NotEnoughMemory)?);
    Ok(())
}

/// # Errors
/// This function will return an error if called without having previously
/// initialized the buddy allocator, which would lead it to be unable to
/// allocate slabs.
#[allow(static_mut_refs)]
pub fn init_slab_allocator(buddy_allocator: &mut BuddyAllocator, slab_allocator: &mut SlabAllocator) -> Result<(), KmallocError> {
    for conf in SLAB_CONFIGS {
        let slab_allocator_addr = buddy_allocator.alloc((PAGE_SIZE * 8) * conf.order).map_err(|_| KmallocError::NotEnoughMemory)?;

        let slab_allocator_addr = NonNull::new(slab_allocator_addr).ok_or(KmallocError::NotEnoughMemory)?;
        // SAFETY:
        // This function is assumed to only ever be called once the buddy allocator is initialized, which
        // would mean that the address we received from it is valid (otherwise we would have gotten an
        // error).
        unsafe { slab_allocator.init_slab_cache(slab_allocator_addr, conf.object_size, 8) };
    }

    Ok(())
}

/// # Errors
/// This function will return an error if any of `init_buddy_allocator` or
/// `init_slab_allocator` fail.
#[allow(static_mut_refs)]
pub fn init() -> Result<(), KmallocError> {
    // SAFETY:
    // We are accessing a static mutable allocator, which is only accessible through this crate. The API
    // of this crate ensures we are not touching it outside of its expected usage.s
    init_buddy_allocator(unsafe { &mut KERNEL_ALLOCATOR.buddy_allocator })?;

    init_slab_allocator(
        // SAFETY:
        // We are accessing a static mutable allocator, which is only accessible through this crate. The API
        // of this crate ensures we are not touching it outside of its expected usage.s
        unsafe { &mut KERNEL_ALLOCATOR.buddy_allocator },
        // SAFETY:
        // We are accessing a static mutable allocator, which is only accessible through this crate. The API
        // of this crate ensures we are not touching it outside of its expected usage.s
        unsafe { &mut KERNEL_ALLOCATOR.slab_allocator },
    )?;

    Ok(())
}
