use crate::{
    printkln,
    vmm::{
        allocators::{
            backend::{
                buddy_allocator::BuddyAllocator,
                slab_allocator::{Slab, SlabAllocationError},
            },
            kmalloc::state::*,
        },
        paging::{
            Access, Permissions,
            mmap::{Mode, mmap},
        },
    },
};

mod state;

#[derive(Debug)]
pub enum KmallocError {
    NotEnoughMemory,
}

#[derive(Debug)]
pub enum KfreeError {
    InvalidPointer,
}

pub const BUDDY_ALLOCATOR_SIZE: usize = 1 << 29;
static mut BUDDY_ALLOCATOR: BuddyAllocator = BuddyAllocator::new(core::ptr::null(), BUDDY_ALLOCATOR_SIZE, unsafe { LEVELS });

const SLAB_CACHE_SIZES: [u16; 9] = [8, 16, 32, 64, 128, 256, 512, 1024, 2048];
const PAGES_PER_SLAB_CACHE: usize = 8;

#[derive(Clone, Copy, Debug)]
pub struct SlabCache {
    empty_slabs: *mut Slab,
    partial_slabs: *mut Slab,
    full_slabs: *mut Slab,

    n_slabs: usize,
    object_size: usize,
}

impl SlabCache {
    pub const fn new(object_size: usize) -> Self {
        Self {
            empty_slabs: core::ptr::null_mut(),
            partial_slabs: core::ptr::null_mut(),
            full_slabs: core::ptr::null_mut(),
            n_slabs: 0,
            object_size,
        }
    }

    pub fn add_slab(&mut self, addr: *mut Slab) -> Result<(), SlabAllocationError> {
        assert!(self.object_size != 0, "Called add_slab on uninitialized SlabCache");

        printkln!("head: 0x{:x}", self.empty_slabs as usize);
        printkln!("Address of slab to be added: 0x{:x}", addr as usize);

        let mut head = self.empty_slabs;
        if head.is_null() {
            self.empty_slabs = addr;
            return Ok(());
        }

        while !unsafe { (*head).next().is_null() } {
            head = unsafe { (*head).next() as *mut Slab };
        }

        unsafe { (*head).set_next(addr) };

        Ok(())
    }

    pub fn alloc(&mut self) -> Result<*const u8, SlabAllocationError> {
        let mut from = self.partial_slabs;

        if from.is_null() {
            from = self.empty_slabs;
            unsafe { (*self.empty_slabs).set_next((*self.empty_slabs).next() as *mut Slab) };
            self.partial_slabs = from;
        }

        unsafe { (*from).alloc() }
    }
}

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

        Self { caches: caches }
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

    printkln!(
        "Initialized Buddy Allocator of size 0x{:x}, root memory: 0x{:x}",
        BUDDY_ALLOCATOR_SIZE,
        cache_memory
    );

    let mut bm = unsafe { &mut BUDDY_ALLOCATOR };
    bm.set_root(cache_memory as *const u8);

    let slab_addr = bm.alloc(4096).map_err(|_| KmallocError::NotEnoughMemory)?;

    let mut slab = unsafe { Slab::new(slab_addr, 1024) };
    for idx in 0..16 {
        printkln!("{:?}", slab.header());
        let ptr = slab.alloc().map_err(|_| KmallocError::NotEnoughMemory);
        if ptr.is_err() {
            printkln!("{:?}", ptr);
            return Ok(());
        }
        let ptr = ptr.unwrap();
        printkln!("[{}] 0x{:x}", idx, ptr as usize);
        slab.free(ptr);
        let ptr = slab.alloc().map_err(|_| KmallocError::NotEnoughMemory);
        if ptr.is_err() {
            printkln!("{:?}", ptr);
        }
    }

    Ok(())
}
