use crate::{
    boot::KERNEL_BASE,
};

const PAGE_SIZE :usize = 0x1000;

#[used]
#[unsafe(no_mangle)]
#[allow(clippy::identity_op)]
#[unsafe(link_section = ".data")]
pub static mut USED_PAGES: Bitmap = Bitmap::new();

#[used]
#[unsafe(no_mangle)]
#[allow(clippy::identity_op)]
#[unsafe(link_section = ".data")]
pub static mut KERNEL_PAGE_TABLES: [[PageTableEntry; 1024]; 1024] = [[PageTableEntry::empty();1024]; 1024];

#[used]
#[unsafe(no_mangle)]
#[allow(clippy::identity_op)]
#[unsafe(link_section = ".data")]
pub static mut KERNEL_PAGE_DIRECTORY_TABLE: [PageDirectoryEntry; 1024] = {
    let mut dir : [PageDirectoryEntry; 1024] = [PageDirectoryEntry::from_usize(0); 1024];

    dir[0] = PageDirectoryEntry(0b10000011);

    // Sets mappings temporary so that the kernel is mapped to the upper half of the vm space
    dir[768] = PageDirectoryEntry::from_usize((0 << 22) | 0b10000011);
    dir[769] = PageDirectoryEntry::from_usize((1 << 22) | 0b10000011);
    dir[770] = PageDirectoryEntry::from_usize((2 << 22) | 0b10000011);
    dir[771] = PageDirectoryEntry::from_usize((3 << 22) | 0b10000011);
    dir[772] = PageDirectoryEntry::from_usize((4 << 22) | 0b10000011);
    dir[773] = PageDirectoryEntry::from_usize((5 << 22) | 0b10000011);

    dir
};

unsafe extern "C" {
    #[link_name = "_kernel_end"]
    static KERNEL_END: u8;
}

pub struct Bitmap {
    content: [u8; Self::BIT_MAP_USED_PAGES_SIZE]
}

impl Bitmap {
    const BIT_MAP_USED_PAGES_SIZE :usize =  usize::MAX / PAGE_SIZE / 8;

    pub const fn new() -> Self {
        Bitmap { content: [0; Self::BIT_MAP_USED_PAGES_SIZE] }
    }

    pub fn get(&self, index: usize) -> u8 {
        let page_index_bit = index % 8;
        let page_index_byte = index / 8;
        return self.content[page_index_byte] & 1 << (7 - page_index_bit);
    }

    pub fn set(&mut self, index: usize, used: bool) {
        match used {
            true => {
                let page_index_bit = index % 8;
                let page_index_byte = index / 8;
                self.content[page_index_byte] |= 1 << (7 - page_index_bit);
            },
            false => {
                let page_index_bit = index % 8;
                let page_index_byte = index / 8;
                self.content[page_index_byte] &= !(1 << (7 - page_index_bit));
            }
        }
    }
}

const PAGE_DIRECTORY_LEN: usize = 1024;

#[bitstruct::bitstruct]
struct PageDirectoryEntry {
    address: u20,
    available_4: u4,
    ps: u1,
    available_1: u1,
    accessed: u1,
    cache_disable: u1,
    write_through: u1,
    user_supervisor: u1,
    read_write: u1,
    present: u1,
}

impl PageDirectoryEntry {
    pub const fn empty() -> Self {
        unsafe { core::mem::transmute::<usize, PageDirectoryEntry>(0) } 
    }
    
    pub const fn from_usize(value: usize) -> Self {
        unsafe { core::mem::transmute::<usize, PageDirectoryEntry>(value) } 
    }

    pub const fn to_usize(&self) -> usize {
        unsafe { core::mem::transmute::<PageDirectoryEntry, usize>(*self) } 
    }
}

const PAGE_TABLE_LEN: usize = 1024;

#[bitstruct::bitstruct]
struct PageTableEntry {
    address: u20,
    available: u3,
    global: u1,
    page_attribute_table: u1,
    dirty: u1,
    accessed: u1,
    cache_disable: u1,
    write_through: u1,
    user_supervisor: u1,
    read_write: u1,
    present: u1,
}

impl PageTableEntry {
    pub const fn empty() -> Self {
        unsafe { core::mem::transmute::<usize, Self>(0) } 
    }
    
    pub const fn from_usize(value: usize) -> Self {
        unsafe { core::mem::transmute::<usize, Self>(value) } 
    }

    pub const fn to_usize(&self) -> usize {
        unsafe { core::mem::transmute::<Self, usize>(*self) } 
    }
}

fn invalidate(vaddr: usize) {
    unsafe { core::arch::asm!("invlpg [{}]", in(reg) vaddr) };
}

#[allow(static_mut_refs)]
pub fn init_memory(_mem_high: usize, _physical_alloc_start: usize) {
    let kernel_end = unsafe {&KERNEL_END as *const _} as usize;
    let kernel_pages_needed = ((kernel_end + 1) - KERNEL_BASE) / PAGE_SIZE;
    
    for i in 0..kernel_pages_needed {
        unsafe { 
            USED_PAGES.set(i, true); 
        }

        let dir_index = i / PAGE_TABLE_LEN;
        let page_index = i % PAGE_TABLE_LEN;
        let mut e = PageTableEntry::default();
        e.set_address(i as u32); 
        e.set_read_write(1);
        e.set_present(1);

        unsafe {
            KERNEL_PAGE_TABLES[dir_index][page_index] = e;
        }
    }

    let mut kernel_page_entries_physical_address = &raw const KERNEL_PAGE_TABLES as usize;
    kernel_page_entries_physical_address -= KERNEL_BASE;


    for i in 0..=(kernel_pages_needed / PAGE_TABLE_LEN) {
        let mut e = PageDirectoryEntry::empty();
        e.set_address((kernel_page_entries_physical_address / PAGE_SIZE) as u32 + i as u32);
        e.set_read_write(1);
        e.set_present(1);

        unsafe {
            KERNEL_PAGE_DIRECTORY_TABLE[768 + i] = e;
        }
    }

    unsafe { KERNEL_PAGE_DIRECTORY_TABLE[0] = PageDirectoryEntry::empty() };

    // Remove maps from caches
    invalidate(0);

    for i in 0..=kernel_pages_needed {
        invalidate(KERNEL_BASE + i * PAGE_SIZE);
    }
}


enum MmapError {
    VaddrAlreadyMapped,
    NotEnoughMemory,

}

fn page_get(vaddr: usize) -> Option<*const PageTableEntry> {
    return Some(unsafe {&KERNEL_PAGE_TABLES[0][0]});
}

pub fn mmap(vaddr: usize, size: usize) -> Result<usize, MmapError> {
    // Check if the current vaddr with the len is already in use
    let pages_needed = (size + (PAGE_SIZE - (size % PAGE_SIZE))) / PAGE_SIZE;
    for _ in 0..pages_needed {
        if let Some(_) = page_get(vaddr) {
            return Err(MmapError::VaddrAlreadyMapped);
        }
    }

    // Check if there is enough pmem

    // Create the mappings in the kernel table

    // set in USED_PAGES

    // return vaddr from the start
    Ok(0)
}