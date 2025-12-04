use crate::{
    printkln,
    terminal::SCREEN,
    vmm::{
        allocators::{
            backend::{
                buddy_allocator::BuddyAllocator,
                slab_allocator::{Slab, SlabAllocationError, SlabFreeError},
            },
            kmalloc::state::*,
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

pub const BUDDY_ALLOCATOR_SIZE: usize = 1 << 22;
static mut BUDDY_ALLOCATOR: BuddyAllocator = BuddyAllocator::new(None, BUDDY_ALLOCATOR_SIZE, unsafe { LEVELS });

// const SLAB_CACHE_SIZES: [u16; 1] = [1024];
const SLAB_CACHE_SIZES: [u16; 9] = [8, 16, 32, 64, 128, 256, 512, 1024, 2048];
const PAGES_PER_SLAB_CACHE: usize = 8;

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

    pub fn add_slab(&mut self, mut addr: NonNull<Slab>, x: usize) -> Result<(), SlabAllocationError> {
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
        for x in 0..n_slabs {
            self.caches[slab_cache_index]
                .add_slab(addr.cast::<Slab>(), x)
                .map_err(|_| KmallocError::NotEnoughMemory)?;

            addr = unsafe { addr.add(PAGE_SIZE) };
        }

        Ok(())
    }
}

pub struct KernelAllocator {
    buddy_allocator: BuddyAllocator,
    slab_allocator: SlabAllocator,
}

#[allow(static_mut_refs)]
pub fn kfree(addr: *const u8) -> Result<(), KfreeError> {
    unsafe { BUDDY_ALLOCATOR.free(addr) }
}

#[allow(static_mut_refs)]
pub fn kmalloc(size: usize) -> Result<*mut u8, KmallocError> {
    unsafe { BUDDY_ALLOCATOR.alloc(size).map_err(|_| KmallocError::NotEnoughMemory) }
}

#[allow(static_mut_refs)]
pub fn init() -> Result<(), KmallocError> {
    let cache_memory = mmap(None, BUDDY_ALLOCATOR_SIZE, Permissions::ReadWrite, Access::Root, Mode::Continous).map_err(|_| KmallocError::NotEnoughMemory)?;

    // let buddy_allocator = unsafe { &mut BUDDY_ALLOCATOR };
    // buddy_allocator.set_root(NonNull::new(cache_memory as *mut u8).ok_or(KmallocError::NotEnoughMemory)?);

    // let mut sa = SlabAllocator::default();

    printkln!(
        "Address range allocated by mmap: 0x{:x}-0x{:x}, Size: 0x{:x}",
        cache_memory,
        cache_memory + BUDDY_ALLOCATOR_SIZE,
        BUDDY_ALLOCATOR_SIZE
    );
    printkln!("Address of VGA buffer: 0x{:x}", unsafe { SCREEN.buffer.as_ptr() as usize });
    let addr = (cache_memory + 259 * PAGE_SIZE) as *mut u8;
    unsafe { *(addr as *mut u8) = 0x00 };
    printkln!("Offending address: 0x{:x}", addr as usize);
    // for addr in (cache_memory as usize)..(cache_memory as usize + 258 * PAGE_SIZE) {}

    // for (idx, size) in SLAB_CACHE_SIZES.iter().enumerate() {
    //     let slab_allocator_addr = buddy_allocator.alloc(PAGE_SIZE * 8).map_err(|_| KmallocError::NotEnoughMemory)?;

    //     let slab_allocator_addr = NonNull::new(slab_allocator_addr).ok_or(KmallocError::NotEnoughMemory)?;
    //     unsafe { sa.init_slab_cache(slab_allocator_addr, *size as usize, 8) }?;

    //     printkln!("{:?}", sa.caches[idx]);
    // }

    Ok(())
}
