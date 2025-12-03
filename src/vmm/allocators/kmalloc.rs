use crate::{
    printkln,
    vmm::{
        allocators::{
            backend::{
                buddy_allocator::BuddyAllocator,
                slab_allocator::{Slab, SlabAllocationError, SlabFreeError},
            },
            kmalloc::state::*,
        },
        paging::{
            Access, Permissions,
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

pub const BUDDY_ALLOCATOR_SIZE: usize = 1 << 27;
static mut BUDDY_ALLOCATOR: BuddyAllocator = BuddyAllocator::new(core::ptr::null(), BUDDY_ALLOCATOR_SIZE, unsafe { LEVELS });

const SLAB_CACHE_SIZES: [u16; 1] = [1024];
// const SLAB_CACHE_SIZES: [u16; 9] = [8, 16, 32, 64, 128, 256, 512, 1024, 2048];
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

    pub fn add_slab(&mut self, addr: NonNull<Slab>) -> Result<(), SlabAllocationError> {
        assert!(self.object_size != 0, "Called add_slab on uninitialized SlabCache");

        let last = self.empty_slabs.into_iter().last();

        self.n_slabs += 1;

        match last {
            None => self.empty_slabs.set_head(Some(addr)),
            Some(last) => unsafe { (*last.as_ptr()).set_next(addr) },
        }

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
        let mut idx = 0;

        while idx < SLAB_CACHE_SIZES.len() {
            caches[idx] = SlabCache::new(SLAB_CACHE_SIZES[idx] as usize);
            idx += 1;
        }

        Self { caches }
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
pub fn kmalloc(size: usize) -> Result<*const u8, KmallocError> {
    unsafe { BUDDY_ALLOCATOR.alloc(size).map_err(|_| KmallocError::NotEnoughMemory) }
}

#[allow(static_mut_refs)]
pub fn init() -> Result<(), KmallocError> {
    let cache_memory = mmap(None, BUDDY_ALLOCATOR_SIZE, Permissions::ReadWrite, Access::Root, Mode::Continous).map_err(|_| KmallocError::NotEnoughMemory)?;

    let buddy_allocator = unsafe { &mut BUDDY_ALLOCATOR };
    buddy_allocator.set_root(cache_memory as *const u8);

    let mut sa = SlabAllocator::default();

    let slab_addr = buddy_allocator.alloc(4096).map_err(|_| KmallocError::NotEnoughMemory)?;
    let mut slab = unsafe { Slab::new(slab_addr, 1024) };

    sa.caches[0]
        .add_slab(NonNull::new(&mut slab as *mut Slab).ok_or(KmallocError::NotEnoughMemory)?)
        .map_err(|_| KmallocError::NotEnoughMemory)?;

    let slab_addr = buddy_allocator.alloc(4096).map_err(|_| KmallocError::NotEnoughMemory)?;
    let mut slab = unsafe { Slab::new(slab_addr, 1024) };
    sa.caches[0]
        .add_slab(NonNull::new(&mut slab as *mut Slab).ok_or(KmallocError::NotEnoughMemory)?)
        .map_err(|_| KmallocError::NotEnoughMemory)?;

    printkln!("{:?}", sa);

    for _ in 0..8 {
        if let Ok(alloc) = sa.caches[0].alloc() {
            printkln!("{:x}", alloc as usize);
            if let Err(e) = sa.caches[0].free(alloc) {
                printkln!("Error freeing {:x}: {:?}", alloc as usize, e);
            }
        } else {
            printkln!("Could not allocate from slab")
        }
    }

    printkln!("{:?}", sa);

    Ok(())
}
