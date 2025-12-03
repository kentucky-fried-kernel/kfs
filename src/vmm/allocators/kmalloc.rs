use crate::{
    printkln,
    vmm::{
        allocators::{
            backend::{
                buddy_allocator::BuddyAllocator,
                slab_allocator::{IntrusiveLink, List, Slab, SlabAllocationError},
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

mod state;

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

// const SLAB_CACHE_SIZES: [u16; 9] = [8, 16, 32, 64, 128, 256, 512, 1024, 2048];
const SLAB_CACHE_SIZES: [u16; 1] = [1024];
const PAGES_PER_SLAB_CACHE: usize = 8;

pub struct SlabListIntoIterator {
    current: Option<NonNull<Slab>>,
}

impl Iterator for SlabListIntoIterator {
    type Item = NonNull<Slab>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.take()?;

        self.current = unsafe { NonNull::new((*current.as_ptr()).next_ptr()?.as_ptr() as *mut Slab) };

        Some(current)
    }

    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let mut last = self.current;

        for slab in self {
            last = Some(slab);
        }

        last
    }
}

impl SlabList {
    pub fn add_back(&mut self, addr: *mut Slab) {
        let last = self.into_iter().last();

        match last {
            None => self.head = NonNull::new(addr),
            Some(last) => unsafe { (*last.as_ptr()).set_next(NonNull::new(addr).expect("let me cook for a minute")) },
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SlabList {
    head: Option<NonNull<Slab>>,
}

impl const Default for SlabList {
    fn default() -> Self {
        Self { head: None }
    }
}

impl IntoIterator for SlabList {
    type IntoIter = SlabListIntoIterator;
    type Item = NonNull<Slab>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter { current: self.head }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SlabCache {
    empty_slabs: List<Slab>,
    partial_slabs: SlabList,
    full_slabs: SlabList,

    n_slabs: usize,
    object_size: usize,
}

impl SlabCache {
    pub const fn new(object_size: usize) -> Self {
        Self {
            empty_slabs: List::<Slab>::default(),
            partial_slabs: SlabList::default(),
            full_slabs: SlabList::default(),
            n_slabs: 0,
            object_size,
        }
    }

    pub fn add_slab(&mut self, addr: NonNull<Slab>) -> Result<(), SlabAllocationError> {
        assert!(self.object_size != 0, "Called add_slab on uninitialized SlabCache");

        match self.empty_slabs.head() {
            Some(head) => printkln!("head: {:?}", head),
            None => printkln!("head: None"),
        }

        printkln!("Address of slab to be added: {:?}", addr);

        let last = self.empty_slabs.into_iter().last();

        self.n_slabs += 1;

        match last {
            None => self.empty_slabs.set_head(addr),
            Some(last) => unsafe { (*last.as_ptr()).set_next(addr) },
        }

        Ok(())
    }

    pub fn alloc(&mut self) -> Result<*const u8, SlabAllocationError> {
        unimplemented!()
        // let mut from = self.partial_slabs;

        // if from.is_null() {
        //     from = self.empty_slabs;
        //     unsafe { (*self.empty_slabs).set_next((*self.empty_slabs).next() as *mut Slab) };
        //     self.partial_slabs = from;
        // }

        // unsafe { (*from).alloc() }
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

    let bm = unsafe { &mut BUDDY_ALLOCATOR };
    bm.set_root(cache_memory as *const u8);

    let mut sa = SlabAllocator::default();
    printkln!("Initialized empty SlabAllocator: {:?}", sa);
    let slab_addr = bm.alloc(4096).map_err(|_| KmallocError::NotEnoughMemory)?;
    let mut slab = unsafe { Slab::new(slab_addr, 1024) };
    printkln!("Initialized empty {:?}", slab);
    sa.caches[0]
        .add_slab(NonNull::new(&mut slab as *mut Slab).unwrap())
        .map_err(|_| KmallocError::NotEnoughMemory)?;
    printkln!("Added slab to {:?}", sa);
    let slab_addr = bm.alloc(4096).map_err(|_| KmallocError::NotEnoughMemory)?;
    let mut slab = unsafe { Slab::new(slab_addr, 1024) };
    sa.caches[0]
        .add_slab(NonNull::new(&mut slab as *mut Slab).unwrap())
        .map_err(|_| KmallocError::NotEnoughMemory)?;
    printkln!("Added slab to {:?}", sa);

    for slab in sa.caches[0].empty_slabs {
        printkln!("{:?}", slab);
    }

    Ok(())
}
