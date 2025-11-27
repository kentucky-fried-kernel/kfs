use crate::{printk, printkln};

const fn generate_cache_sizes() -> [u16; 30] {
    let mut sizes = [0; 30];
    let mut count = 1;
    let mut i = 0;

    sizes[0] = 8;

    while i < 13 && count < 30 {
        let last = sizes[count - 1];

        if i >= 4 && count < 30 {
            sizes[count] = ((last * 5) / 4) & !0x08;
            count += 1;
        }

        if i >= 3 && count < 30 {
            sizes[count] = last + last / 2;
            count += 1;
        }

        if i >= 5 && count < 30 {
            sizes[count] = ((last * 7) / 4) & !0x08;
            count += 1;
        }

        if count < 30 {
            sizes[count] = last * 2;
            count += 1;
        }

        i += 1;
    }

    sizes
}

pub const KMALLOC_ALIGNMENT: usize = 0x08;
pub const STANDARD_CACHE_SIZES: [u16; 30] = generate_cache_sizes();
// 30
const MAX_CACHES: u8 = STANDARD_CACHE_SIZES.len() as u8;
const MAX_SLABS_PER_CACHE: u8 = 64;

static mut MMAP: Mmap = Mmap::empty();
static mut ALLOCATOR: Allocator = Allocator::new();

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

pub struct Allocator {
    caches: [Cache; MAX_CACHES as usize],
}

impl Allocator {
    pub const fn new() -> Self {
        let mut caches = [Cache::new(0); MAX_CACHES as usize];
        let mut idx = 0;

        while idx < MAX_CACHES {
            caches[idx as usize] = Cache::new(STANDARD_CACHE_SIZES[idx as usize] as u16);
            idx += 1;
        }
        Self { caches }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cache {
    slabs: [Option<Slab>; MAX_SLABS_PER_CACHE as usize],
    object_size: u16,
    n_slabs: u8,
}

impl Cache {
    pub const fn new(object_size: u16) -> Self {
        Self {
            slabs: [None; MAX_SLABS_PER_CACHE as usize],
            object_size,
            n_slabs: 0,
        }
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self {
            slabs: [None; MAX_SLABS_PER_CACHE as usize],
            object_size: 0,
            n_slabs: 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Slab {
    // must be page-aligned
    start_addr: *const u8,
    n_free: u16,
    object_size: u16,
    first_object: *const u8,
}

impl Slab {
    pub fn new(object_size: u16, _offset: u16) -> Option<Self> {
        let usable_space_per_slab = 4096 - core::mem::size_of::<Slab>() as u16;
        let remaining_space = usable_space_per_slab % object_size;
        let n_per_page = usable_space_per_slab / object_size;

        printkln!("object_size: {}, n_per_page: {}, remaining_space: {}", object_size, n_per_page, remaining_space);

        #[allow(static_mut_refs)]
        let start_addr = unsafe { MMAP.mmap() }?;

        Some(Self {
            start_addr,
            n_free: n_per_page,
            object_size,
            first_object: start_addr,
        })
    }

    pub const fn at(&self, index: usize) -> Option<*const u8> {
        let offset = index * self.object_size as usize;
        if offset + self.object_size as usize > 0x1000 {
            return None;
        }
        Some(unsafe { self.start_addr.add(offset) })
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

#[allow(static_mut_refs)]
fn kmalloc_cache_create(object_size: u16) -> Result<(), Error> {
    let idx = STANDARD_CACHE_SIZES
        .iter()
        .position(|&x| x == object_size)
        .expect("object_size must be one of STANDARD_CACHE_SIZES");

    let cache = unsafe { &mut ALLOCATOR.caches[idx as usize] };

    if cache.n_slabs == MAX_SLABS_PER_CACHE {
        return Err(Error::NoSpaceLeft);
    }

    match Slab::new(object_size, 0) {
        Some(slab) => {
            cache.slabs[cache.n_slabs as usize] = Some(slab);
            cache.n_slabs += 1;
            Ok(())
        }
        None => Err(Error::MmapFailure),
    }
}

pub fn kmalloc(size: usize) -> Result<*const usize, &'static str> {
    if size > 10240 {
        return Err("Maximum cache size is 10240, dynamic allocation not implemented yet.");
    }

    Ok(size as *const usize)
}

pub fn kfree(addr: *const usize) {}

pub fn ksize(addr: *const usize) -> usize {
    0
}

pub fn init() -> Result<(), Error> {
    kmalloc_cache_create(8)?;
    // kmalloc_cache_create(16)?;

    for cache in unsafe { &ALLOCATOR.caches[..1] } {
        for slab in cache.slabs.iter() {
            if let Some(slab) = slab {
                printkln!("object_size: {}, {:?}", cache.object_size, slab);
            }
        }
    }

    Ok(())
}
