use crate::{
    printkln,
    vmm::{
        allocators::{bitmap::BitMap, buddy::BuddyAllocatorBitmap},
        paging::{
            Access, Permissions,
            mmap::{MmapError, Mode, mmap},
        },
    },
};

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
    bitmap: BitMap<4096>,
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
                self.bitmap.set(idx);
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
                self.bitmap.unset(idx);
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

#[allow(static_mut_refs)]
pub fn kmalloc(size: usize) -> usize {
    let ptr = mmap(None, size, Permissions::Read, Access::Root, Mode::Continous).unwrap();
    printkln!("{:x}", ptr);
    ptr
}

#[allow(static_mut_refs)]
pub fn init() -> Result<(), Error> {
    // unsafe {
    //     CACHE_8 = BlockCache::new(8).unwrap();
    //     CACHE_16 = BlockCache::new(16).unwrap();
    //     CACHE_32 = BlockCache::new(32).unwrap();
    //     CACHE_64 = BlockCache::new(64).unwrap();
    //     CACHE_128 = BlockCache::new(128).unwrap();
    //     CACHE_256 = BlockCache::new(256).unwrap();
    //     CACHE_512 = BlockCache::new(512).unwrap();
    //     CACHE_1024 = BlockCache::new(1024).unwrap();
    //     CACHE_2048 = BlockCache::new(2048).unwrap();
    // }

    let mut bm = BuddyAllocatorBitmap::new(0 as *const u8, 32768);
    printkln!("Allocating 4096 bytes from buddy allocator");
    printkln!("Received address: 0x{:x}", bm.alloc(4096).unwrap() as usize);

    Ok(())
}
