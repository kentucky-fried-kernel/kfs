#![allow(unused)]

use crate::{
    bitmap::BitMap,
    printkln,
    vmm::{
        allocators::{backend::buddy_allocator::BuddyAllocator, kmalloc::state::*},
        paging::{
            Access, Permissions,
            mmap::{MmapError, Mode, mmap},
        },
    },
};

mod state;

#[repr(C, align(0x1000))]
#[derive(Clone, Copy)]
pub struct Page {
    data: [u8; 4096],
}

#[derive(Debug)]
pub enum Error {
    NoSpaceLeft,
    MmapFailure,
    InvalidPointer,
    DoubleFree,
}

const PAGES_PER_CACHE: usize = 8;

#[derive(Clone, Copy, Debug)]
pub struct BlockCache {
    pages: [*const u8; PAGES_PER_CACHE],
    bitmap: BitMap<4096, 8>,
    object_size: u16,
}

impl BlockCache {
    #[allow(static_mut_refs)]
    pub fn new(object_size: u16) -> Result<Self, MmapError> {
        let mut pages = [core::ptr::null::<u8>(); PAGES_PER_CACHE];

        for page in pages.iter_mut() {
            *page = mmap(None, 4096, Permissions::ReadWrite, Access::Root, Mode::Continous)? as *const u8;
        }

        Ok(Self {
            pages,
            bitmap: BitMap::new(),
            object_size,
        })
    }

    pub fn malloc(&mut self) -> Option<*const u8> {
        for ((idx, object), bit) in self.into_iter().enumerate().zip(self.bitmap) {
            if bit == 0 {
                self.bitmap.set(idx, 1);
                return Some(object);
            }
        }
        None
    }

    pub fn free(&mut self, addr: *const u8) -> Result<(), Error> {
        for ((idx, object), bit) in self.into_iter().enumerate().zip(self.bitmap) {
            if object == addr {
                if bit == 0 {
                    return Err(Error::InvalidPointer);
                }
                self.bitmap.clear(idx);
                return Ok(());
            }
        }
        Err(Error::InvalidPointer)
    }
}

impl IntoIterator for BlockCache {
    type Item = *const u8;
    type IntoIter = BlockCacheIntoIterator;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            block_cache: self,
            current_position: self.pages[0],
            page_index: 0,
        }
    }
}

pub struct BlockCacheIntoIterator {
    block_cache: BlockCache,
    current_position: *const u8,
    page_index: usize,
}

impl Iterator for BlockCacheIntoIterator {
    type Item = *const u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_position as usize + self.block_cache.object_size as usize >= (self.current_position as usize & !0xFFF) + 0x1000 {
            if self.page_index == PAGES_PER_CACHE - 1 {
                return None;
            }
            self.page_index += 1;
            self.current_position = self.block_cache.pages[self.page_index];
            return Some(self.current_position);
        }

        let current_position = self.current_position;

        self.current_position = unsafe { self.current_position.add(self.block_cache.object_size as usize) };
        Some(current_position)
    }
}

static mut CACHE_8: BlockCache = unsafe { core::mem::zeroed() };
static mut CACHE_16: BlockCache = unsafe { core::mem::zeroed() };
static mut CACHE_32: BlockCache = unsafe { core::mem::zeroed() };
static mut CACHE_64: BlockCache = unsafe { core::mem::zeroed() };
static mut CACHE_128: BlockCache = unsafe { core::mem::zeroed() };
static mut CACHE_256: BlockCache = unsafe { core::mem::zeroed() };
static mut CACHE_512: BlockCache = unsafe { core::mem::zeroed() };
static mut CACHE_1024: BlockCache = unsafe { core::mem::zeroed() };
static mut CACHE_2048: BlockCache = unsafe { core::mem::zeroed() };

pub const BUDDY_ALLOCATOR_SIZE: usize = 1 << 20;
static mut BUDDY_ALLOCATOR: BuddyAllocator = BuddyAllocator::new(core::ptr::null(), BUDDY_ALLOCATOR_SIZE, unsafe { LEVELS });

#[allow(static_mut_refs)]
pub fn kmalloc(size: usize) -> Option<*const u8> {
    unsafe { BUDDY_ALLOCATOR.alloc(size) }
}

#[allow(static_mut_refs)]
pub fn init() -> Result<(), Error> {
    let cache_memory = mmap(None, BUDDY_ALLOCATOR_SIZE, Permissions::ReadWrite, Access::Root, Mode::Continous).map_err(|_| Error::MmapFailure)?;

    let mut bm = unsafe { &mut BUDDY_ALLOCATOR };
    bm.set_root(cache_memory as *const u8);

    Ok(())
}
