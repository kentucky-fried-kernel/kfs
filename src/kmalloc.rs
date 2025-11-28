use crate::printkln;

pub const KMALLOC_ALIGNMENT: usize = 0x08;

static mut MMAP: Mmap = Mmap::empty();

#[repr(C, align(0x1000))]
#[derive(Clone, Copy)]
pub struct Page {
    data: [u8; 4096],
}

impl Page {
    pub const fn empty() -> Self {
        Self { data: [0; 4096] }
    }
}

pub struct Mmap {
    pages: [Page; 1024],
    bitmap: [u8; 1024 / 8],
}

impl Mmap {
    pub const fn empty() -> Self {
        Self {
            pages: [Page::empty(); 1024],
            bitmap: [0; 1024 / 8],
        }
    }

    pub fn mmap(&mut self) -> Option<*const u8> {
        for i in 0..(1024 / 8) {
            if self.bitmap[i] == 0xff {
                continue;
            }

            for j in 0..8 {
                if (self.bitmap[i] >> j) & 1 == 0 {
                    self.bitmap[i] |= 1 << j;
                    return Some(self.pages[i * 8 + j].data.as_ptr());
                }
            }
        }
        None
    }

    pub fn munmap(&mut self, addr: *const u8) -> Result<(), ()> {
        for i in 0..1024 {
            if self.pages[i].data.as_ptr() == addr {
                self.bitmap[i / 8] &= !(1 << (i % 8));
                return Ok(());
            }
        }
        Err(())
    }
}

#[derive(Debug)]
pub enum Error {
    NoSpaceLeft,
    MmapFailure,
}

pub fn kmalloc(size: usize) -> Result<*const usize, &'static str> {
    Ok(size as *const usize)
}

pub fn kfree(addr: *const usize) {}

pub fn ksize(addr: *const usize) -> usize {
    0
}

// basic approach:
// (merge branch with whole memory mapped to avoid page faults)
// start with pre-mapping block caches for object sizes 32, 64, 256, 512, 1024, 2048
// service requests <= 2048 bytes by finding the first free block
// allocate whole pages for larger requests
// stop overengineering from the start!!

const PAGES_PER_CACHE: usize = 8;

#[derive(Clone, Copy, Debug)]
pub struct BlockCache {
    pages: [*const u8; PAGES_PER_CACHE],
    bitmap: u8,
    object_size: u16,
}

impl BlockCache {
    #[allow(static_mut_refs)]
    pub fn new(object_size: u16) -> Result<Self, Error> {
        Ok(Self {
            pages: [unsafe { MMAP.mmap() }.ok_or(Error::MmapFailure)?; PAGES_PER_CACHE],
            bitmap: 0,
            object_size,
        })
    }
}

impl IntoIterator for BlockCache {
    type Item = *const u8;
    type IntoIter = BlockCacheIntoIterator;

    fn into_iter(self) -> Self::IntoIter {
        BlockCacheIntoIterator {
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

        self.current_position = unsafe { self.current_position.add(self.page_index * self.block_cache.object_size as usize) };
        Some(self.current_position)
    }
}

#[allow(static_mut_refs)]
pub fn init() -> Result<(), Error> {
    let bc = BlockCache::new(16);

    // unsafe { *(0xd0000000 as *mut usize) = 0 };
    for b in bc.into_iter() {
        printkln!("{:?}", b);
    }

    Ok(())
}
