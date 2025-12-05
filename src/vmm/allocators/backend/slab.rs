use core::ptr::NonNull;

use crate::vmm::{
    allocators::kmalloc::{IntrusiveLink, KmallocError, List},
    paging::PAGE_SIZE,
};

const SLAB_HEADER_OVERHEAD: usize = (size_of::<SlabHeader>() & !(0x08 - 1)) + 0x08;

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
    pub const fn new(object_size: usize) -> Self {
        Self {
            empty_slabs: List::<Slab>::default(),
            partial_slabs: List::<Slab>::default(),
            full_slabs: List::<Slab>::default(),
            n_slabs: 0,
            object_size,
        }
    }

    pub fn add_slab(&mut self, mut addr: NonNull<Slab>) -> Result<(), SlabAllocationError> {
        assert!(self.object_size != 0, "Called add_slab on uninitialized SlabCache");

        self.empty_slabs.add_front(&mut addr);
        self.n_slabs += 1;

        Ok(())
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
    pub unsafe fn init_slab_cache(&mut self, addr: NonNull<u8>, object_size: usize, n_slabs: usize) -> Result<(), KmallocError> {
        let slab_cache_index = SLAB_CACHE_SIZES
            .iter()
            .position(|x| *x as usize == object_size)
            .expect("Called SlabAllocator::init_slab_cache with an invalid object_size");

        let mut addr = addr;
        for _ in 0..n_slabs {
            self.caches[slab_cache_index]
                .add_slab(addr.cast::<Slab>())
                .map_err(|_| KmallocError::NotEnoughMemory)?;

            addr = unsafe { addr.add(PAGE_SIZE) };
        }

        Ok(())
    }

    pub fn caches(&self) -> &[SlabCache] {
        &self.caches
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

#[derive(Clone, Copy, Debug)]
pub struct SlabHeader {
    object_size: usize,
    /// Tracks the number of allocated objects in this slab. Allows prioritizing
    /// allocations from fuller slabs to maximize the number of empty slabs to give
    /// back to the main allocator in case of memory pressure.
    allocated: usize,
    next: Option<NonNull<Payload>>,
}

impl IntrusiveLink for Slab {
    #[inline]
    fn next_ptr(&self) -> Option<NonNull<Self>>
    where
        Self: Sized,
    {
        self.next
    }

    #[inline]
    fn next_ptr_mut(&mut self) -> &mut Option<NonNull<Self>>
    where
        Self: Sized,
    {
        &mut self.next
    }
}

/// Order 0 Slab.
// TODO: add different slab orders:
// Order 0: spans one contiguous page (8 - 256 bytes objects)
// Order 1: spans four contiguous pages (512 - 1024 bytes)
// Order 2: spans eight pages (2048+ bytes)
#[derive(Clone, Copy, Debug)]
pub struct Slab {
    addr: *const u8,
    next: Option<NonNull<Slab>>,
}

impl Slab {
    /// Creates a `Slab` object from `addr` and `object_size`.
    /// # Safety
    /// It is the caller's responsibility to ensure that `addr` points to a valid,
    /// page-aligned address, with at least 0x1000 read-writable bytes.
    pub unsafe fn new(addr: *const u8, object_size: usize) -> Self {
        assert!(addr.is_aligned_to(PAGE_SIZE), "addr is not page-aligned");
        assert!(object_size >= size_of::<*const u8>(), "object_size must be large enough to hold a pointer");

        let header: *mut SlabHeader = addr as *mut SlabHeader;

        let objects_start_addr = unsafe { addr.add(SLAB_HEADER_OVERHEAD) };
        let header_overhead = objects_start_addr as usize - addr as usize;

        let available_space = PAGE_SIZE - header_overhead;
        let n_objects = available_space / object_size;

        assert!(n_objects > 0, "object_size is too large for a single page slab");

        unsafe {
            (*header).allocated = 0;
            (*header).object_size = object_size;
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
            (*header).next = NonNull::new(objects_start_addr as *mut Payload);
        }

        Self { addr, next: None }
    }

    #[inline]
    pub fn address(&self) -> *const u8 {
        self.addr
    }

    #[inline]
    pub fn set_next(&mut self, next: NonNull<Slab>) {
        self.next = Some(next);
    }

    #[inline]
    pub fn header(&self) -> &SlabHeader {
        let header_ptr = self.addr as *const SlabHeader;

        // SAFETY: The constructor guarantees self.addr points to a valid page
        // where the SlabHeader is correctly initialized at the start.
        unsafe { &*header_ptr }
    }

    fn header_mut(&mut self) -> &mut SlabHeader {
        let header_ptr = self.addr as *mut SlabHeader;

        // SAFETY: The constructor guarantees self.addr points to a valid page
        // where the SlabHeader is correctly initialized at the start.
        unsafe { &mut (*header_ptr) }
    }

    #[inline]
    pub fn full(&self) -> bool {
        self.header().allocated == self.max_objects()
    }

    #[inline]
    fn max_objects(&self) -> usize {
        (PAGE_SIZE - SLAB_HEADER_OVERHEAD) / self.header().object_size
    }

    /// Returns a pointer to a free memory region of size `object_size`, or a
    /// `SlabAllocationError` if no more space is left.
    pub fn alloc(&mut self) -> Result<*mut u8, SlabAllocationError> {
        if self.header().allocated == self.max_objects() {
            return Err(SlabAllocationError::NotEnoughMemory);
        }

        let allocation = {
            let header = self.header_mut();
            let allocation = header.next.ok_or(SlabAllocationError::NotEnoughMemory)?;

            header.next = unsafe { *allocation.as_ptr() }.next;
            header.allocated += 1;
            allocation
        };

        Ok(allocation.as_ptr() as *mut u8)
    }

    pub fn free(&mut self, addr: *const u8) -> Result<(), SlabFreeError> {
        assert!(
            addr >= self.addr && addr < (self.addr as usize + 0x1000) as *const u8,
            "addr is out of range for this slab"
        );

        let header = self.header_mut();

        let next = header.next;

        header.next = NonNull::new(addr as *mut Payload);
        header.allocated -= 1;

        unsafe { (*(addr as *mut Payload)).next = next };

        Ok(())
    }
}
