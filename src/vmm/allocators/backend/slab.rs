use core::ptr::NonNull;

use crate::{
    expect_opt,
    vmm::{
        allocators::kmalloc::{IntrusiveLink, KfreeError, KmallocError, List},
        paging::PAGE_SIZE,
    },
};

// const SLAB_HEADER_OVERHEAD: usize = (size_of::<Order0Slab>() & !(0x08 - 1)) + 0x08;

pub const SLAB_CACHE_SIZES: [u16; 9] = [8, 16, 32, 64, 128, 256, 512, 1024, 2048];

#[derive(Clone, Copy, Debug)]
pub struct SlabCache<S: SlabOps> {
    empty_slabs: List<S>,
    partial_slabs: List<S>,
    full_slabs: List<S>,

    n_slabs: usize,
    object_size: usize,
}

impl<S: SlabOps> SlabCache<S>
where
    S: Copy,
{
    #[must_use]
    pub const fn new(object_size: usize) -> Self {
        Self {
            empty_slabs: List::<S>::default(),
            partial_slabs: List::<S>::default(),
            full_slabs: List::<S>::default(),
            n_slabs: 0,
            object_size,
        }
    }

    /// # Safety
    /// If any of the following conditions are violated, the result is Undefined
    /// Behavior:
    /// * `addr` must point to a valid allocation of **at least** `PAGE_SIZE` bytes.
    unsafe fn add_slab(&mut self, mut addr: NonNull<S>) {
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
pub enum SlabCacheType {
    Order0(SlabCache<Slab<{ 1 << 0 }>>),
    Order1(SlabCache<Slab<{ 1 << 1 }>>),
    Order2(SlabCache<Slab<{ 1 << 2 }>>),
    Order3(SlabCache<Slab<{ 1 << 3 }>>),
}

impl SlabCacheType {
    pub const fn new(object_size: usize, order: usize) -> Self {
        match order {
            1 => Self::Order0(SlabCache::new(object_size)),
            2 => Self::Order1(SlabCache::new(object_size)),
            4 => Self::Order2(SlabCache::new(object_size)),
            8 => Self::Order3(SlabCache::new(object_size)),
            _ => panic!("Unsupported slab order"),
        }
    }

    pub fn alloc(&mut self) -> Result<*mut u8, SlabAllocationError> {
        match self {
            Self::Order0(cache) => cache.alloc(),
            Self::Order1(cache) => cache.alloc(),
            Self::Order2(cache) => cache.alloc(),
            Self::Order3(cache) => cache.alloc(),
        }
    }

    pub fn free(&mut self, addr: *const u8) -> Result<(), SlabFreeError> {
        match self {
            Self::Order0(cache) => cache.free(addr),
            Self::Order1(cache) => cache.free(addr),
            Self::Order2(cache) => cache.free(addr),
            Self::Order3(cache) => cache.free(addr),
        }
    }
}

#[derive(Debug)]
pub struct SlabAllocator {
    caches: [SlabCacheType; SLAB_CACHE_SIZES.len()],
}

struct SlabConfig {
    pub object_size: usize,
    pub order: usize,
}

pub const SLAB_CONFIGS: [SlabConfig; 9] = [
    SlabConfig { object_size: 8, order: 1 },
    SlabConfig { object_size: 16, order: 1 },
    SlabConfig { object_size: 32, order: 1 },
    SlabConfig { object_size: 64, order: 1 },
    SlabConfig { object_size: 128, order: 1 },
    SlabConfig { object_size: 256, order: 1 },
    SlabConfig { object_size: 512, order: 2 },
    SlabConfig { object_size: 1024, order: 2 },
    SlabConfig { object_size: 2048, order: 1 },
];

impl const Default for SlabAllocator {
    fn default() -> Self {
        let caches = [
            SlabCacheType::new(SLAB_CONFIGS[0].object_size, SLAB_CONFIGS[0].order),
            SlabCacheType::new(SLAB_CONFIGS[1].object_size, SLAB_CONFIGS[1].order),
            SlabCacheType::new(SLAB_CONFIGS[2].object_size, SLAB_CONFIGS[2].order),
            SlabCacheType::new(SLAB_CONFIGS[3].object_size, SLAB_CONFIGS[3].order),
            SlabCacheType::new(SLAB_CONFIGS[4].object_size, SLAB_CONFIGS[4].order),
            SlabCacheType::new(SLAB_CONFIGS[5].object_size, SLAB_CONFIGS[5].order),
            SlabCacheType::new(SLAB_CONFIGS[6].object_size, SLAB_CONFIGS[6].order),
            SlabCacheType::new(SLAB_CONFIGS[7].object_size, SLAB_CONFIGS[7].order),
            SlabCacheType::new(SLAB_CONFIGS[8].object_size, SLAB_CONFIGS[8].order),
        ];

        Self { caches }
    }
}

impl SlabAllocator {
    /// # Safety
    /// If any of the following conditions are violated, the result is Undefined
    /// Behavior:
    /// * `addr` must point to a valid allocation of **at least** `PAGE_SIZE * n_slabs` bytes.
    pub unsafe fn init_slab_cache(&mut self, addr: NonNull<u8>, object_size: usize, n_slabs: usize) {
        let config_idx = SLAB_CONFIGS.iter().position(|x| x.object_size == object_size);
        let config_idx = expect_opt!(config_idx, "Called SlabAllocator::init_slab_cache with an invalid object_size");

        let order = SLAB_CONFIGS[config_idx].order;
        let slab_size = order * PAGE_SIZE;

        let mut addr = addr;
        for _ in 0..n_slabs {
            match &mut self.caches[config_idx] {
                SlabCacheType::Order0(cache) => {
                    let slab_ptr = addr.cast().as_ptr();
                    // SAFETY:
                    // We are calling `Slab::init`, which is unsafe since it initializes memory
                    // in-place, which the compiler cannot verify. If
                    // `init_slab_cache`'s # Safety directive was followed, `slab_ptr` points to
                    // valid memory which we can safely write to.
                    unsafe { Slab::<1>::init(slab_ptr, object_size) };
                    // SAFETY:
                    // Assuming the Safety directive of this function was followed, he address we are passing to
                    // `add_slab` is guaranteed to be a valid allocation due to the bounds of this loop.
                    unsafe { cache.add_slab(addr.cast()) };
                }
                SlabCacheType::Order1(cache) => {
                    let slab_ptr = addr.cast().as_ptr();
                    // SAFETY:
                    // We are calling `Slab::init`, which is unsafe since it initializes memory
                    // in-place, which the compiler cannot verify. If
                    // `init_slab_cache`'s # Safety directive was followed, `slab_ptr` points to
                    // valid memory which we can safely write to.
                    unsafe { Slab::<2>::init(slab_ptr, object_size) };
                    // SAFETY:
                    // Assuming the Safety directive of this function was followed, he address we are passing to
                    // `add_slab` is guaranteed to be a valid allocation due to the bounds of this loop.
                    unsafe { cache.add_slab(addr.cast()) };
                }
                SlabCacheType::Order2(cache) => {
                    let slab_ptr = addr.cast().as_ptr();
                    // SAFETY:
                    // We are calling `Slab::init`, which is unsafe since it initializes memory
                    // in-place, which the compiler cannot verify. If
                    // `init_slab_cache`'s # Safety directive was followed, `slab_ptr` points to
                    // valid memory which we can safely write to.
                    unsafe { Slab::<4>::init(slab_ptr, object_size) };
                    // SAFETY:
                    // Assuming the Safety directive of this function was followed, he address we are passing to
                    // `add_slab` is guaranteed to be a valid allocation due to the bounds of this loop.
                    unsafe { cache.add_slab(addr.cast()) };
                }
                SlabCacheType::Order3(cache) => {
                    let slab_ptr = addr.cast().as_ptr();
                    // SAFETY:
                    // We are calling `Slab::init`, which is unsafe since it initializes memory
                    // in-place, which the compiler cannot verify. If
                    // `init_slab_cache`'s # Safety directive was followed, `slab_ptr` points to
                    // valid memory which we can safely write to.
                    unsafe { Slab::<8>::init(slab_ptr, object_size) };
                    // SAFETY:
                    // Assuming the Safety directive of this function was followed, he address we are passing to
                    // `add_slab` is guaranteed to be a valid allocation due to the bounds of this loop.
                    unsafe { cache.add_slab(addr.cast()) };
                }
            }
            // SAFETY:
            // `PAGE_SIZE` does not overflow `isize`, and `addr` points to a valid
            // allocation at least `PAGE_SIZE * n_slabs` if this function's
            // safety directive was respected.
            addr = unsafe { addr.add(slab_size) };
        }
    }

    #[must_use]
    pub fn caches(&self) -> &[SlabCacheType] {
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
        for cache in &mut self.caches {
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

pub trait SlabOps: IntrusiveLink + Sized {
    /// How many contiguous pages this slab spans
    const ORDER: usize;

    /// Total size of this slab in bytes
    const SLAB_SIZE: usize = PAGE_SIZE * Self::ORDER;

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
    unsafe fn init(slab_ptr: *mut Self, object_size: usize);

    #[must_use]
    fn address(&self) -> *const u8;

    #[must_use]
    fn full(&self) -> bool;

    fn max_objects(&self) -> usize;

    fn object_size(&self) -> usize;

    fn allocated(&self) -> usize;

    /// Returns a pointer to a free memory region of size `object_size`, or a
    /// `SlabAllocationError` if no more space is left.
    ///
    /// # Errors
    /// Returns a `SlabAllocationError` if allocation is not possible due to insufficient memory.
    fn alloc(&mut self) -> Result<*mut u8, SlabAllocationError>;

    /// # Errors
    /// Returns a `SlabFreeError` if `addr` does not point to memory managed by this `Slab` object.
    fn free(&mut self, addr: *const u8) -> Result<(), SlabFreeError>;
}

#[derive(Clone, Copy, Debug)]
pub struct Slab<const ORDER: usize> {
    /// Intrusive list link for `SlabCache` lists (empty/partial/full)
    list_next: Option<NonNull<Slab<ORDER>>>,
    /// Size of each object in this slab
    object_size: usize,
    /// Number of currently allocated objects
    allocated: usize,
    /// Free list head - points to the next available object
    free_list_next: Option<NonNull<Payload>>,
}

impl<const ORDER: usize> IntrusiveLink for Slab<ORDER> {
    #[inline]
    fn next_ptr(&self) -> Option<NonNull<Self>> {
        self.list_next
    }

    #[inline]
    fn next_ptr_mut(&mut self) -> &mut Option<NonNull<Self>> {
        &mut self.list_next
    }
}

impl<const ORDER: usize> SlabOps for Slab<ORDER> {
    const ORDER: usize = ORDER;

    #[inline]
    fn object_size(&self) -> usize {
        self.object_size
    }

    #[inline]
    fn allocated(&self) -> usize {
        self.allocated
    }

    #[inline]
    fn full(&self) -> bool {
        self.allocated == self.max_objects()
    }

    #[inline]
    fn address(&self) -> *const u8 {
        (self as *const Self).cast()
    }

    fn alloc(&mut self) -> Result<*mut u8, SlabAllocationError> {
        if self.allocated == self.max_objects() {
            return Err(SlabAllocationError::NotEnoughMemory);
        }

        let allocation = self.free_list_next.ok_or(SlabAllocationError::NotEnoughMemory)?;

        // SAFETY: If this `Slab` was initialized according to its safety documentation,
        // `allocation` is guaranteed to be usable memory that we can safely access.
        self.free_list_next = unsafe { *allocation.as_ptr() }.next;
        self.allocated += 1;

        Ok(allocation.as_ptr().cast())
    }

    #[allow(clippy::cast_ptr_alignment)]
    fn free(&mut self, addr: *const u8) -> Result<(), SlabFreeError> {
        debug_assert!(addr.is_aligned_to(8));

        let slab_end = (self.address() as usize + Self::SLAB_SIZE) as *const u8;
        if addr < self.address() || addr > slab_end {
            return Err(SlabFreeError::InvalidPointer);
        }

        let next = self.free_list_next;
        self.free_list_next = NonNull::new(addr as *mut Payload);
        self.allocated -= 1;

        // SAFETY: If this `Slab` was initialized according to its safety documentation,
        // `addr` is guaranteed to be memory owned by this slab that we can safely access.
        unsafe { (*(addr as *mut Payload)).next = next };

        Ok(())
    }

    unsafe fn init(slab_ptr: *mut Self, object_size: usize) {
        let addr = slab_ptr.cast::<u8>();
        debug_assert!(addr.is_aligned_to(PAGE_SIZE), "addr is not page-aligned");
        debug_assert!(object_size >= 8, "object_size must be at least 8");

        const fn slab_header_overhead<const ORDER: usize>() -> usize {
            (size_of::<Slab<ORDER>>() & !(0x08 - 1)) + 0x08
        }

        let header_overhead = slab_header_overhead::<ORDER>();

        // SAFETY: `header_overhead` does not overflow `isize` and `addr` points to valid memory
        let objects_start_addr = unsafe { addr.add(header_overhead) };

        let available_space = Self::SLAB_SIZE - header_overhead;
        let n_objects = available_space / object_size;

        debug_assert!(n_objects > 0, "object_size is too large for order {} slab", ORDER);

        // SAFETY: According to this function's safety documentation, `slab_ptr` must point
        // to a valid allocation that we can safely access.
        #[allow(clippy::multiple_unsafe_ops_per_block)]
        unsafe {
            (*slab_ptr).list_next = None;
            (*slab_ptr).object_size = object_size;
            (*slab_ptr).allocated = 0;
        }

        // Initialize the free list
        let mut current_obj_ptr = objects_start_addr;
        for i in 0..n_objects {
            // SAFETY: Loop is bounded to n_objects which guarantees we stay within allocation
            let next_obj_ptr = unsafe { current_obj_ptr.add(object_size) };

            #[allow(clippy::cast_ptr_alignment)]
            let link_ptr = current_obj_ptr.cast::<*const u8>();
            let value = if i == n_objects - 1 { core::ptr::null() } else { next_obj_ptr };

            // SAFETY: link_ptr is guaranteed to be in valid allocation
            unsafe { *link_ptr = value };

            current_obj_ptr = next_obj_ptr;
        }

        // SAFETY: slab_ptr is guaranteed to be valid by function's Safety docs
        #[allow(clippy::cast_ptr_alignment)]
        unsafe {
            (*slab_ptr).free_list_next = NonNull::new(objects_start_addr.cast());
        }
    }

    fn max_objects(&self) -> usize {
        const fn slab_header_overhead<const ORDER: usize>() -> usize {
            (size_of::<Slab<ORDER>>() & !(0x08 - 1)) + 0x08
        }

        (Self::SLAB_SIZE - slab_header_overhead::<ORDER>()) / self.object_size
    }
}
