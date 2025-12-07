use core::ptr::NonNull;

use crate::{
    expect_opt,
    vmm::{
        allocators::kmalloc::{IntrusiveLink, KfreeError, KmallocError, List},
        paging::PAGE_SIZE,
    },
};

const SLAB_HEADER_OVERHEAD: usize = (size_of::<Slab>() & !(0x08 - 1)) + 0x08;

pub const SLAB_CACHE_SIZES: [u16; 9] = [8, 16, 32, 64, 128, 256, 512, 1024, 2048];

#[derive(Clone, Copy, Debug)]
pub struct SlabCache {
    empty_slabs: List<Slab>,
    partial_slabs: List<Slab>,
    full_slabs: List<Slab>,

    n_slabs: usize,
    object_size: usize,
}

impl SlabCache {
    #[must_use]
    pub const fn new(object_size: usize) -> Self {
        Self {
            empty_slabs: List::<Slab>::default(),
            partial_slabs: List::<Slab>::default(),
            full_slabs: List::<Slab>::default(),
            n_slabs: 0,
            object_size,
        }
    }

    /// # Safety
    /// If any of the following conditions are violated, the result is Undefined
    /// Behavior:
    /// * `addr` must point to a valid allocation of **at least** `PAGE_SIZE` bytes.
    unsafe fn add_slab(&mut self, mut addr: NonNull<Slab>) {
        assert!(self.object_size != 0, "Called add_slab on uninitialized SlabCache");

        // SAFETY:
        // This function's Safety contract enforces that `addr` point to a valid allocation of
        // at least `PAGE_SIZE` bytes.
        unsafe { self.empty_slabs.add_front(&mut addr) };
        self.n_slabs += 1;
    }

    fn alloc(&mut self) -> Result<*mut u8, SlabAllocationError> {
        match (self.partial_slabs.head(), self.empty_slabs.head()) {
            (Some(mut slab), _) => {
                // SAFETY:
                // We are calling `as_mut()` on `slab`, which cannot be null due to its type.
                // The slabs themselves are initialized by the `SlabAllocator`,
                // which ensures that each allocation is sucessful before
                // considering using it as a slab.
                let allocation = unsafe { slab.as_mut() }.alloc();
                // SAFETY:
                // We are calling `as_ref()` on `slab`, which cannot be null due to its type.
                // The slabs themselves are initialized by the `SlabAllocator`,
                // which ensures that each allocation is sucessful before
                // considering using it as a slab.
                if unsafe { slab.as_ref() }.full() {
                    // SAFETY:
                    // We are calling `add_front()` on `self.full_slabs`, which is guaranteed to point
                    // to a valid node that will not be deallocated for the lifetime of this `SlabCache`.
                    unsafe {
                        self.full_slabs
                            .add_front(&mut self.partial_slabs.take_head().ok_or(SlabAllocationError::NotEnoughMemory)?);
                    };
                }
                allocation
            }
            (_, Some(mut slab)) => {
                // SAFETY:
                // We are calling `as_mut()` on `slab`, which cannot be null due to its type.
                // The slabs themselves are initialized by the `SlabAllocator`,
                // which ensures that each allocation is sucessful before
                // considering using it as a slab.
                let allocation = unsafe { slab.as_mut() }.alloc();
                let mut head = self.empty_slabs.take_head().ok_or(SlabAllocationError::NotEnoughMemory)?;
                // SAFETY:
                // We are calling `add_front()` on `self.partial_slabs`, which is guaranteed to point
                // to a valid node that will not be deallocated for the lifetime of this `SlabCache`.
                unsafe { self.partial_slabs.add_front(&mut head) };
                allocation
            }
            _ => Err(SlabAllocationError::NotEnoughMemory),
        }
    }

    // Freeing is currently very slow, need to find a clean way for the slabs to be
    // sorted by address for O(logn) lookups.
    fn free(&mut self, addr: *const u8) -> Result<(), SlabFreeError> {
        for mut slab in self.partial_slabs {
            // SAFETY:
            // We are calling `as_mut()` on `slab`, which cannot be null due to its type.
            // The slabs themselves are initialized by the `SlabAllocator`,
            // which ensures that each allocation is sucessful before
            // considering using it as a slab.
            if let Ok(()) = unsafe { slab.as_mut() }.free(addr) {
                return Ok(());
            }
        }
        for mut slab in self.full_slabs {
            // SAFETY:
            // We are calling `as_mut()` on `slab`, which cannot be null due to its type.
            // The slabs themselves are initialized by the `SlabAllocator`,
            // which ensures that each allocation is sucessful before
            // considering using it as a slab.
            if let Ok(()) = unsafe { slab.as_mut() }.free(addr) {
                return Ok(());
            }
        }
        Err(SlabFreeError::InvalidPointer)
    }
}

#[derive(Debug)]
pub struct SlabAllocator {
    caches: [SlabCache; SLAB_CACHE_SIZES.len()],
}

impl const Default for SlabAllocator {
    fn default() -> Self {
        let mut caches = [SlabCache::new(0); SLAB_CACHE_SIZES.len()];
        let mut cache_idx = 0;

        while cache_idx < SLAB_CACHE_SIZES.len() {
            caches[cache_idx] = SlabCache::new(SLAB_CACHE_SIZES[cache_idx] as usize);
            cache_idx += 1;
        }

        Self { caches }
    }
}

impl SlabAllocator {
    /// # Safety
    /// If any of the following conditions are violated, the result is Undefined
    /// Behavior:
    /// * `addr` must point to a valid allocation of **at least** `PAGE_SIZE * n_slabs` bytes.
    pub unsafe fn init_slab_cache(&mut self, addr: NonNull<u8>, object_size: usize, n_slabs: usize) {
        let slab_cache_index = SLAB_CACHE_SIZES.iter().position(|x| *x as usize == object_size);
        let slab_cache_index = expect_opt!(slab_cache_index, "Called SlabAllocator::init_slab_cache with an invalid object_size");

        let mut addr = addr;
        for _ in 0..n_slabs {
            let slab_ptr = addr.cast::<Slab>().as_ptr();
            // SAFETY:
            // We are calling `Slab::init`, which is unsafe since it initializes memory
            // in-place, which the compiler cannot verify. If
            // `init_slab_cache`'s # Safety directive was followed, `slab_ptr` points to
            // valid memory which we can safely write to.
            unsafe { Slab::init(slab_ptr, object_size) };
            // SAFETY:
            // Assuming the Safety directive of this function was followed, he address we are passing to
            // `add_slab` is guaranteed to be a valid allocation due to the bounds of this loop.
            unsafe { self.caches[slab_cache_index].add_slab(addr.cast()) };

            // SAFETY:
            // `PAGE_SIZE` does not overflow `isize`, and `addr` points to a valid
            // allocation at least `PAGE_SIZE * n_slabs` if this function's
            // safety directive was respected.
            addr = unsafe { addr.add(PAGE_SIZE) };
        }
    }

    #[must_use]
    pub fn caches(&self) -> &[SlabCache] {
        &self.caches
    }

    /// # Errors
    /// This function will return an error if allocation fails due to
    /// insufficient memory.
    pub fn alloc(&mut self, size: usize) -> Result<*mut u8, KmallocError> {
        let slab_cache_index = if size <= 8 {
            0
        } else {
            let index = SLAB_CACHE_SIZES
                .iter()
                .map_windows(|[x, y]| size > **x as usize && size <= **y as usize)
                .position(|x| x);
            expect_opt!(index, "Called SlabAllocator::alloc with an invalid size") + 1
        };

        self.caches[slab_cache_index].alloc().map_err(|_| KmallocError::NotEnoughMemory)
    }

    /// # Errors
    /// This function will return an error if `addr` points to a memory address
    /// not managed by this `SlabAllocator`.
    pub fn free(&mut self, addr: *const u8) -> Result<(), KfreeError> {
        for mut cache in self.caches {
            if cache.free(addr).is_ok() {
                return Ok(());
            }
        }
        Err(KfreeError::InvalidPointer)
    }
}

#[repr(u8)]
pub enum SlabObjectStatus {
    Free = 0,
    Allocated = 1,
}

#[derive(Debug)]
pub enum SlabAllocationError {
    NotEnoughMemory,
}

#[derive(Debug)]
pub enum SlabFreeError {
    InvalidPointer,
}

#[derive(Clone, Copy, Debug)]
pub struct Payload {
    next: Option<NonNull<Payload>>,
}

impl IntrusiveLink for Slab {
    #[inline]
    fn next_ptr(&self) -> Option<NonNull<Self>>
    where
        Self: Sized,
    {
        self.list_next
    }

    #[inline]
    fn next_ptr_mut(&mut self) -> &mut Option<NonNull<Self>>
    where
        Self: Sized,
    {
        &mut self.list_next
    }
}

/// Order 0 Slab.
/// This struct is stored at the beginning of each slab page and contains both
/// the intrusive list link (for `SlabCache` lists) and the free list management
/// data.
// TODO: add different slab orders:
// Order 0: spans one contiguous page (8 - 256 bytes objects)
// Order 1: spans four contiguous pages (512 - 1024 bytes)
// Order 2: spans eight pages (2048+ bytes)
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Slab {
    /// Intrusive list link for `SlabCache` lists (empty/partial/full)
    list_next: Option<NonNull<Slab>>,
    /// Size of each object in this slab
    object_size: usize,
    /// Number of currently allocated objects
    allocated: usize,
    /// Free list head - points to the next available object
    free_list_next: Option<NonNull<Payload>>,
}

impl Slab {
    /// Initializes a slab in place at the given address.
    ///
    /// # Safety
    /// If any of the following conditions are violated, the result is Undefined
    /// Behavior:
    /// * `slab_ptr` must point to a page-aligned allocation of **at least** `0x1000` bytes.
    ///
    /// # Panics
    /// This function will panic if called with wrong arguments, like a
    /// `slab_ptr` which is not page-aligned.
    pub unsafe fn init(slab_ptr: *mut Slab, object_size: usize) {
        let addr = slab_ptr.cast::<u8>();
        assert!(addr.is_aligned_to(PAGE_SIZE), "addr is not page-aligned");
        assert!(object_size >= 8, "object_size must be at least 8");

        // SAFETY:
        // * `SLAB_HEADER_OVERHEAD` is a constant that does not overflow `isize`
        // * According to this function's Safety docs, `addr` must point to a valid allocation that we can
        //   safely access
        let objects_start_addr = unsafe { addr.add(SLAB_HEADER_OVERHEAD) };
        let header_overhead = objects_start_addr as usize - addr as usize;

        let available_space = PAGE_SIZE - header_overhead;
        let n_objects = available_space / object_size;

        assert!(n_objects > 0, "object_size is too large for a single page slab");

        // SAFETY:
        // According to this function's safety documentation, `slab_ptr` must point
        // to a valid allocation of at least `0x1000` bytes that we can safely access.
        #[allow(clippy::multiple_unsafe_ops_per_block)]
        unsafe {
            (*slab_ptr).list_next = None;
            (*slab_ptr).object_size = object_size;
            (*slab_ptr).allocated = 0;
        }

        let mut current_obj_ptr = objects_start_addr;

        for i in 0..n_objects {
            // SAFETY:
            // According to this function's safety documentation, `slab_ptr` must point
            // to a valid allocation of at least `0x1000` bytes that we can safely access.
            // The loop is bounded to `n_objects`, which guarantees that address after
            // `slab_ptr + 0x1000` will be accessed.
            let next_obj_ptr = unsafe { current_obj_ptr.add(object_size) };

            // We are casting `*const u8` to a more strictly aligned pointer
            // (`*mut *const u8`), however we know that `current_obj_ptr` is
            // at least 8-bytes aligned due to the restrictions enforced at
            // the beginning of this function.
            #[allow(clippy::cast_ptr_alignment)]
            let link_ptr = current_obj_ptr.cast::<*const u8>();
            let value = if i == n_objects - 1 { core::ptr::null() } else { next_obj_ptr };

            // SAFETY:
            // We are dereferencing `link_ptr`, which is guaranteed to be in a valid
            // allocation by this function's Safety docs and the bounds of this
            // loop.
            unsafe { *link_ptr = value };

            current_obj_ptr = next_obj_ptr;
        }

        // We are casting `*const u8` to a more strictly aligned pointer
        // (`*mut *const u8`), however we know that `current_obj_ptr` is
        // at least 8-bytes aligned due to the restrictions enforced at
        // the beginning of this function.
        //
        // SAFETY:
        // We are dereferencing `slab_ptr`, which is guaranteed to be in a valid
        // allocation by this function's Safety docs.
        #[allow(clippy::cast_ptr_alignment)]
        unsafe {
            (*slab_ptr).free_list_next = NonNull::new(objects_start_addr.cast());
        }
    }

    #[inline]
    #[must_use]
    pub fn address(&self) -> *const u8 {
        (self as *const Slab).cast()
    }

    #[inline]
    pub fn set_next(&mut self, next: NonNull<Slab>) {
        self.list_next = Some(next);
    }

    #[inline]
    #[must_use]
    pub fn full(&self) -> bool {
        self.allocated == self.max_objects()
    }

    #[inline]
    fn max_objects(&self) -> usize {
        (PAGE_SIZE - SLAB_HEADER_OVERHEAD) / self.object_size
    }

    /// Returns a pointer to a free memory region of size `object_size`, or a
    /// `SlabAllocationError` if no more space is left.
    fn alloc(&mut self) -> Result<*mut u8, SlabAllocationError> {
        if self.allocated == self.max_objects() {
            return Err(SlabAllocationError::NotEnoughMemory);
        }

        let allocation = self.free_list_next.ok_or(SlabAllocationError::NotEnoughMemory)?;

        // SAFETY:
        // If this `Slab` was intialized according to its safety documentation,
        // `allocation` is guaranteed to be usable memory that we can safely
        // access.
        self.free_list_next = unsafe { *allocation.as_ptr() }.next;
        self.allocated += 1;

        Ok(allocation.as_ptr().cast())
    }

    // We cast from `*const u8` to more strictly aligned pointers (`*mut Payload`),
    // however the assertion in the beginning of the function ensures that no
    // pointer is passed that is not at least 8-bytes aligned.
    #[allow(clippy::cast_ptr_alignment)]
    fn free(&mut self, addr: *const u8) -> Result<(), SlabFreeError> {
        assert!(addr.is_aligned_to(8));

        if addr < self.address() || addr > (self.address() as usize + PAGE_SIZE) as *const u8 {
            return Err(SlabFreeError::InvalidPointer);
        }

        let next = self.free_list_next;

        self.free_list_next = NonNull::new(addr as *mut Payload);
        self.allocated -= 1;

        // SAFETY:
        // If this `Slab` was intialized according to its safety documentation,
        // `addr` is guaranteed to be memory owned by this slab that we can safely
        // access.
        unsafe { (*(addr as *mut Payload)).next = next };

        Ok(())
    }
}
