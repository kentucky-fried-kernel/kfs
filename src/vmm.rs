use core::mem::take;

use crate::{boot::{KERNEL_BASE, MemoryMap}, printkln};

pub const PAGE_SIZE: usize = 0x1000;
pub const MEMORY_MAX: usize = usize::MAX;

#[used]
#[unsafe(no_mangle)]
#[allow(clippy::identity_op)]
#[unsafe(link_section = ".data")]
pub static mut USED_PAGES: [Option<Access>; USED_PAGES_SIZE] = [None; USED_PAGES_SIZE];
const USED_PAGES_SIZE: usize = MEMORY_MAX / PAGE_SIZE;

#[used]
#[unsafe(no_mangle)]
#[allow(clippy::identity_op)]
#[unsafe(link_section = ".data")]
pub static mut KERNEL_PAGE_TABLES: [[PageTableEntry; 1024]; 1024] = [[PageTableEntry::empty(); 1024]; 1024];

#[used]
#[unsafe(no_mangle)]
#[allow(clippy::identity_op)]
#[unsafe(link_section = ".data")]
pub static mut KERNEL_PAGE_DIRECTORY_TABLE: [PageDirectoryEntry; KERNEL_PAGE_DIRECTORY_TABLE_SIZE] = {
    let mut dir: [PageDirectoryEntry; KERNEL_PAGE_DIRECTORY_TABLE_SIZE] = [PageDirectoryEntry::from_usize(0); KERNEL_PAGE_DIRECTORY_TABLE_SIZE];

    dir[0] = PageDirectoryEntry(0b10000011);

    // Sets mappings temporary so that the kernel is mapped to the upper half of the vm space
    dir[768] = PageDirectoryEntry::from_usize((0 << 22) | 0b10000011);
    dir[769] = PageDirectoryEntry::from_usize((1 << 22) | 0b10000011);
    dir[770] = PageDirectoryEntry::from_usize((2 << 22) | 0b10000011);
    dir[771] = PageDirectoryEntry::from_usize((3 << 22) | 0b10000011);
    dir[772] = PageDirectoryEntry::from_usize((4 << 22) | 0b10000011);
    dir[773] = PageDirectoryEntry::from_usize((5 << 22) | 0b10000011);
    dir[774] = PageDirectoryEntry::from_usize((6 << 22) | 0b10000011);
    dir[775] = PageDirectoryEntry::from_usize((7 << 22) | 0b10000011);

    dir
};
const KERNEL_PAGE_DIRECTORY_TABLE_SIZE: usize = 1024;

unsafe extern "C" {
    #[link_name = "_kernel_end"]
    static KERNEL_END: u8;
}

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
    let kernel_end = unsafe { &KERNEL_END as *const _ } as usize;
    let kernel_pages_needed = ((kernel_end + 1) - KERNEL_BASE) / PAGE_SIZE;

    for i in 0..kernel_pages_needed {
        unsafe {
            USED_PAGES[i] = Some(Access::Root);
        }

        let dir_index = i / PAGE_TABLE_LEN;
        let page_index = i % PAGE_TABLE_LEN;
        let mut e = PageTableEntry::empty();
        e.set_address(i as u32);
        e.set_read_write(1);
        e.set_present(1);

        unsafe {
            KERNEL_PAGE_TABLES[768 + dir_index][page_index] = e;
        }
    }

    let mut kernel_page_entries_physical_address = &raw const KERNEL_PAGE_TABLES as usize;
    kernel_page_entries_physical_address -= KERNEL_BASE;

    for i in 0..=(kernel_pages_needed / PAGE_TABLE_LEN) {
        let mut e = PageDirectoryEntry::empty();
        e.set_address((kernel_page_entries_physical_address / PAGE_SIZE) as u32 + i as u32 + 768);
        e.set_read_write(1);
        e.set_present(1);

        unsafe {
            KERNEL_PAGE_DIRECTORY_TABLE[768 + i] = e;
        }
    }
    // Set all page direcotry entries to empty but connect them to the page tables
    for i in 0..KERNEL_PAGE_DIRECTORY_TABLE_SIZE {
        unsafe {
            let already_set = KERNEL_PAGE_DIRECTORY_TABLE[i].address() != 0;
            if already_set {
                continue;
            }
        }
        let mut e = PageDirectoryEntry::empty();
        e.set_address((kernel_page_entries_physical_address / PAGE_SIZE) as u32 + i as u32);

        unsafe {
            KERNEL_PAGE_DIRECTORY_TABLE[i] = e;
        }
    }

    unsafe { KERNEL_PAGE_DIRECTORY_TABLE[0] = PageDirectoryEntry::empty() };

    // Remove maps from caches
    invalidate(0);

    for i in 0..=kernel_pages_needed {
        invalidate(KERNEL_BASE + i * PAGE_SIZE);
    }
}

#[derive(Debug)]
pub enum MmapError {
    VaddrRangeAlreadyMapped,
    NotEnoughMemory,
    NotImplemented,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Permissions {
    Read = 0,
    ReadWrite = 1,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Access {
    User,
    Root,
}

fn page_get(vaddr: usize) -> Option<*const PageTableEntry> {
    return Some(unsafe { &KERNEL_PAGE_TABLES[0][0] });
}

#[allow(static_mut_refs)]
pub fn mmap(vaddr: Option<usize>, size: usize, permissions: Permissions, access: Access) -> Result<usize, MmapError> {
    if let Some(_) = vaddr {
        unimplemented!();
    }

    let pages_needed = (PAGE_SIZE + size - 1) / PAGE_SIZE;

    unsafe {
        let page_physical_used_lowest_index_user = match USED_PAGES.iter().rev().enumerate().find(|(i, p)| p.is_some_and(|p| p == Access::User)) {
            Some((i, _)) => i,
            None => u32::MAX as usize / PAGE_SIZE,
        };
        let page_physical_used_highest_index_root = match USED_PAGES.iter().enumerate().find(|(i, p)| p.is_some_and(|p| p == Access::Root)) {
            Some((i, _)) => i,
            None => 0,
        };

        match access {
            Access::Root => {
                unimplemented!()
            }
            Access::User => {
                let free_pages_physical_iter = USED_PAGES
                    .iter()
                    .enumerate()
                    .rev()
                    .take(USED_PAGES.len() - page_physical_used_highest_index_root)
                    .filter(|(_, b)| b.is_none())
                    .take(pages_needed);

                let free_pages = free_pages_physical_iter.count();
                if free_pages < pages_needed {
                    return Err(MmapError::NotImplemented);
                }

                let free_pages_physical_iter = USED_PAGES
                    .iter_mut()
                    .enumerate()
                    .rev()
                    .take(USED_PAGES.len() - page_physical_used_highest_index_root)
                    .filter(|(_, b)| b.is_none())
                    .take(pages_needed);
                let pages_physical_to_be_allocated = free_pages_physical_iter.take(pages_needed);

                let pages_virtual_free_iter = KERNEL_PAGE_TABLES.iter_mut().flatten().enumerate()
                ;
                let pages_virtual_to_be_allocated = {
                    let mut res = None;
                    for (i, p) in pages_virtual_free_iter {
                        printkln!("i: {}", i);
                        let all_are_free = pages_needed == KERNEL_PAGE_TABLES.iter_mut().flatten().skip(i).take(pages_needed).filter(|p| p.present() == 0).count();
                        if all_are_free {
                            res = Some(KERNEL_PAGE_TABLES.iter_mut().flatten().enumerate().skip(i));
                            break;
                        }
                    }
                    res
                };

                let pages_virtual_to_be_allocated = match pages_virtual_to_be_allocated {
                    Some(it) => it,
                    None => return Err(MmapError::VaddrRangeAlreadyMapped),
                };

                let pages_to_be_allocated = pages_physical_to_be_allocated.zip(pages_virtual_to_be_allocated);

                let mut return_value  = Err(MmapError::NotEnoughMemory);
                for ((physical_i, physical_page), (virtual_i, virtual_page)) in pages_to_be_allocated {
                    printkln!("Physical: 0x{:x}", physical_i);
                    printkln!("Virtual: 0x{:x}", virtual_i);
                    
                    *physical_page = Some(access);
                    let mut e = PageTableEntry::empty();
                    e.set_address(physical_i as u32);
                    e.set_read_write(permissions as u8);
                    e.set_present(1);
                    *virtual_page = e;
                    if let Err(_) = return_value {
                        return_value = Ok(virtual_i * PAGE_SIZE);
                    }
                }

                return return_value;
            },
        }
    }
    // Check if there is enough pmem

    // Create the mappings in the kernel table

    // set in USED_PAGES

    // return vaddr from the start
    Ok(0)
}
