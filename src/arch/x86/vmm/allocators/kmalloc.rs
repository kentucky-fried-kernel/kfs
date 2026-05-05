use core::{alloc::GlobalAlloc, ptr::NonNull};

mod list;
mod state;

pub use list::{IntrusiveLink, List};

use crate::{
    _kernel_end,
    arch::x86::vmm::{
        allocators::backend::{
            buddy::{BUDDY_ALLOCATOR_SIZE, BuddyAllocator},
            slab::{SLAB_CONFIGS, SlabAllocator},
        },
        init::mmap_init,
        page::PAGE_SIZE,
    },
    boot::KERNEL_BASE,
    buddy_allocator_levels,
};

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
    pub slabs_start: usize,
    pub slabs_end: usize,
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
        let size = layout.size().max(layout.align());

        kmalloc(size).unwrap_or_default()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        // SAFETY:
        // Passing a random pointer to `kfree` would result in undefined behavior, but since we rely
        // on rustc to insert all allocation/free operations, we can safely assume that no
        // invalid pointers will be passed to this function.
        assert!(unsafe { kfree(ptr) }.is_ok());
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
    slabs_start: 0,
    slabs_end: 0,
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
    slabs_start: 0,
    slabs_end: 0,
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
    // We are accessing a static mutable allocator, which is only accessible through this crate.
    // The API of this crate ensures we are not touching it outside of its expected usage.
    let allocator = unsafe { &mut KERNEL_ALLOCATOR };
    let addr_usize = addr as usize;

    if addr_usize >= allocator.slabs_start && addr_usize < allocator.slabs_end {
        allocator.slab_allocator.free(addr)
    } else {
        allocator.buddy_allocator.free(addr)
    }
}

/// # Errors
/// This function will return an error if it fails to find a sufficiently large
/// block of memory for the allocation.
#[allow(static_mut_refs)]
pub fn kmalloc(size: usize) -> Result<*mut u8, KmallocError> {
    // SAFETY:
    // We are accessing a static mutable allocator, which is only accessible through this crate.
    // The API of this crate ensures we are not touching it outside of its expected usage.
    let allocator = unsafe { &mut KERNEL_ALLOCATOR };

    match size {
        0..=2048 => allocator.slab_allocator.alloc(size).map_err(|_| KmallocError::NotEnoughMemory),
        2049.. => allocator
            .buddy_allocator
            .alloc(1 << ((size - 1).ilog2() + 1))
            .map_err(|_| KmallocError::NotEnoughMemory),
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

#[inline]
const fn align_up(x: usize, align: usize) -> usize {
    (x + align - 1) & !(align - 1)
}

/// # Errors
/// This function will return an error if the initial allocation for the
/// `BuddyAllocator` (made via `mmap`) fails.
#[allow(static_mut_refs)]
pub fn init_buddy_allocator(allocator: &mut KernelAllocator) -> Result<(), KmallocError> {
    let kernel_end_vaddr: usize = &raw const _kernel_end as usize;

    // The buddy allocator requires its root to be naturally aligned to its
    // total size — addresses inside the tree are derived from `root` purely
    // by offset, so a misaligned root would have it hand out addresses
    // outside the region (or with the buddy of the last block landing in
    // unmapped memory). Round up.
    let vaddr = align_up(kernel_end_vaddr, BUDDY_ALLOCATOR_SIZE);
    let paddr = vaddr - KERNEL_BASE;

    mmap_init(vaddr as *mut u8, Some(paddr as *mut u8), BUDDY_ALLOCATOR_SIZE).map_err(|()| KmallocError::NotEnoughMemory)?;

    allocator
        .buddy_allocator
        .set_root(NonNull::new(vaddr as *mut u8).ok_or(KmallocError::NotEnoughMemory)?);

    Ok(())
}

/// # Errors
/// This function will return an error if called without having previously
/// initialized the buddy allocator, which would lead it to be unable to
/// allocate slabs.
#[allow(static_mut_refs)]
pub fn init_slab_allocator(allocator: &mut KernelAllocator) -> Result<(), KmallocError> {
    const SLABS_PER_CACHE: usize = 32;

    let total_size = SLAB_CONFIGS.iter().fold(0, |acc, conf| acc + PAGE_SIZE * conf.order * SLABS_PER_CACHE);

    let kernel_end_vaddr: usize = &raw const _kernel_end as usize;
    // Slabs go right after the buddy region. The buddy region is aligned to
    // its own size, so its end is also naturally aligned to BUDDY_ALLOCATOR_SIZE
    // — far more than the slab region needs. PAGE_SIZE alignment is enough here.
    let buddy_end = align_up(kernel_end_vaddr, BUDDY_ALLOCATOR_SIZE) + BUDDY_ALLOCATOR_SIZE;
    let vaddr = buddy_end; // already page-aligned (BUDDY_ALLOCATOR_SIZE is a power of 2 ≥ PAGE_SIZE)
    let paddr = vaddr - KERNEL_BASE;

    mmap_init(vaddr as *mut u8, Some(paddr as *mut u8), total_size).map_err(|()| KmallocError::NotEnoughMemory)?;

    allocator.slabs_start = vaddr;
    allocator.slabs_end = vaddr + total_size;

    let mut cur = vaddr as *mut u8;
    for conf in SLAB_CONFIGS {
        let slab_cache_addr = NonNull::new(cur).ok_or(KmallocError::NotEnoughMemory)?;
        unsafe { allocator.slab_allocator.init_slab_cache(slab_cache_addr, conf.object_size, SLABS_PER_CACHE) };
        let slab_size_bytes = PAGE_SIZE * conf.order * SLABS_PER_CACHE;
        cur = unsafe { cur.add(slab_size_bytes) };
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
    init_buddy_allocator(unsafe { &mut KERNEL_ALLOCATOR })?;

    init_slab_allocator(
        // SAFETY:
        // We are accessing a static mutable allocator, which is only accessible through this crate.
        // The API of this crate ensures we are not touching it outside of its expected usage.s
        unsafe { &mut KERNEL_ALLOCATOR },
    )?;

    Ok(())
}
