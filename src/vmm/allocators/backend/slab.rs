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

    /// # Panics
    /// This function will panic if called on an uninitialized `SlabCache`.
    pub fn add_slab(&mut self, mut addr: NonNull<Slab>) {
        assert!(self.object_size != 0, "Called add_slab on uninitialized SlabCache");

        self.empty_slabs.add_front(&mut addr);
        self.n_slabs += 1;
    }

    pub fn alloc(&mut self) -> Result<*mut u8, SlabAllocationError> {
        match (self.partial_slabs.head(), self.empty_slabs.head()) {
            (Some(mut slab), _) => {
                let allocation = unsafe { slab.as_mut() }.alloc();
                if unsafe { slab.as_ref() }.full() {
                    self.full_slabs
                        .add_front(&mut self.partial_slabs.take_head().ok_or(SlabAllocationError::NotEnoughMemory)?);
                }
                allocation
            }
            (_, Some(mut slab)) => {
                let allocation = unsafe { slab.as_mut() }.alloc();
                let mut head = self.empty_slabs.take_head().ok_or(SlabAllocationError::NotEnoughMemory)?;
                self.partial_slabs.add_front(&mut head);
                allocation
            }
            _ => Err(SlabAllocationError::NotEnoughMemory),
        }
    }

    // Freeing is currently very slow, need to find a clean way for the slabs to be
    // sorted by address for O(logn) lookups.
    pub fn free(&mut self, addr: *const u8) -> Result<(), SlabFreeError> {
        for mut slab in self.partial_slabs {
            match unsafe { slab.as_mut() }.free(addr) {
                Ok(_) => return Ok(()),
                Err(_) => continue,
            }
        }
        for mut slab in self.full_slabs {
            match unsafe { slab.as_mut() }.free(addr) {
                Ok(_) => return Ok(()),
                Err(_) => continue,
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
    /// It is the caller's responsibility to ensure that `addr` points to a valid, allocated memory address,
    /// containing **at least** `PAGE_SIZE * n_slabs` read-writable bytes.
    pub unsafe fn init_slab_cache(&mut self, addr: NonNull<u8>, object_size: usize, n_slabs: usize) {
        let slab_cache_index = SLAB_CACHE_SIZES.iter().position(|x| *x as usize == object_size);
        let slab_cache_index = expect_opt!(slab_cache_index, "Called SlabAllocator::init_slab_cache with an invalid object_size");

        let mut addr = addr;
        for _ in 0..n_slabs {
            let slab_ptr = addr.cast::<Slab>().as_ptr();
            unsafe { Slab::init(slab_ptr, object_size) };
            self.caches[slab_cache_index].add_slab(addr.cast());

            addr = unsafe { addr.add(PAGE_SIZE) };
        }
    }

    pub fn caches(&self) -> &[SlabCache] {
        &self.caches
    }

    /// # Safety
    /// This function handles raw pointers. It is the caller's responsibility to ensure
    /// that the `Slab`s stored in this `SlabCache` object are properly initialized.
    pub unsafe fn alloc(&mut self, size: usize) -> Result<*mut u8, KmallocError> {
        let slab_cache_index = if size <= 8 {
            0
        } else {
            SLAB_CACHE_SIZES
                .iter()
                .map_windows(|[x, y]| size > **x as usize && size <= **y as usize)
                .position(|x| x)
                .expect("Called SlabAllocator::alloc with an invalid size")
                + 1
        };

        self.caches[slab_cache_index].alloc().map_err(|_| KmallocError::NotEnoughMemory)
    }

    /// # Safety
    /// This function handles raw pointers. It is the caller's responsibility to ensure
    /// that the `Slab`s stored in this `SlabCache` object are properly initialized.
    pub unsafe fn free(&mut self, addr: *const u8) -> Result<(), KfreeError> {
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

impl From<u8> for SlabObjectStatus {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Free,
            1 => Self::Allocated,
            _ => unreachable!(),
        }
    }
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
/// the intrusive list link (for SlabCache lists) and the free list management data.
// TODO: add different slab orders:
// Order 0: spans one contiguous page (8 - 256 bytes objects)
// Order 1: spans four contiguous pages (512 - 1024 bytes)
// Order 2: spans eight pages (2048+ bytes)
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Slab {
    /// Intrusive list link for SlabCache lists (empty/partial/full)
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
    /// It is the caller's responsibility to ensure that `slab_ptr` points to a valid,
    /// page-aligned address, with at least `0x1000` reserved bytes.
    pub unsafe fn init(slab_ptr: *mut Slab, object_size: usize) {
        let addr = slab_ptr as *const u8;
        assert!(addr.is_aligned_to(PAGE_SIZE), "addr is not page-aligned");
        assert!(object_size >= size_of::<*const u8>(), "object_size must be large enough to hold a pointer");

        let objects_start_addr = unsafe { addr.add(SLAB_HEADER_OVERHEAD) };
        let header_overhead = objects_start_addr as usize - addr as usize;

        let available_space = PAGE_SIZE - header_overhead;
        let n_objects = available_space / object_size;

        assert!(n_objects > 0, "object_size is too large for a single page slab");

        unsafe {
            (*slab_ptr).list_next = None;
            (*slab_ptr).object_size = object_size;
            (*slab_ptr).allocated = 0;
        }

        let mut current_obj_ptr = objects_start_addr;

        for i in 0..n_objects {
            let next_obj_ptr = unsafe { current_obj_ptr.add(object_size) };

            unsafe {
                let link_ptr = current_obj_ptr as *mut *const u8;

                if i == n_objects - 1 {
                    *link_ptr = core::ptr::null();
                } else {
                    *link_ptr = next_obj_ptr;
                }
            }

            current_obj_ptr = next_obj_ptr;
        }

        unsafe {
            (*slab_ptr).free_list_next = NonNull::new(objects_start_addr as *mut Payload);
        }
    }

    #[inline]
    pub fn address(&self) -> *const u8 {
        self as *const Slab as *const u8
    }

    #[inline]
    pub fn set_next(&mut self, next: NonNull<Slab>) {
        self.list_next = Some(next);
    }

    #[inline]
    pub fn full(&self) -> bool {
        self.allocated == self.max_objects()
    }

    #[inline]
    fn max_objects(&self) -> usize {
        (PAGE_SIZE - SLAB_HEADER_OVERHEAD) / self.object_size
    }

    /// Returns a pointer to a free memory region of size `object_size`, or a
    /// `SlabAllocationError` if no more space is left.
    pub fn alloc(&mut self) -> Result<*mut u8, SlabAllocationError> {
        if self.allocated == self.max_objects() {
            return Err(SlabAllocationError::NotEnoughMemory);
        }

        let allocation = self.free_list_next.ok_or(SlabAllocationError::NotEnoughMemory)?;

        self.free_list_next = unsafe { *allocation.as_ptr() }.next;
        self.allocated += 1;

        Ok(allocation.as_ptr() as *mut u8)
    }

    pub fn free(&mut self, addr: *const u8) -> Result<(), SlabFreeError> {
        if addr < self.address() || addr > (self.address() as usize + PAGE_SIZE) as *const u8 {
            return Err(SlabFreeError::InvalidPointer);
        }
        let next = self.free_list_next;

        self.free_list_next = NonNull::new(addr as *mut Payload);
        self.allocated -= 1;

        unsafe { (*(addr as *mut Payload)).next = next };

        Ok(())
    }
}
