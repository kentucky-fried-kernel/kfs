use core::arch::asm;

use crate::{
    boot::{KERNEL_BASE, MultibootInfo, MultibootMmapEntry},
    printkln,
    vmm::paging::{
        Access, PAGE_SIZE,
        page_entries::{PageDirectoryEntry, PageTableEntry},
        state::{KERNEL_PAGE_DIRECTORY_TABLE, KERNEL_PAGE_TABLES, PAGE_TABLE_SIZE, USED_PAGES},
    },
};

unsafe extern "C" {
    #[link_name = "_kernel_end"]
    pub static KERNEL_END: u8;
}

fn invalidate(vaddr: usize) {
    unsafe { core::arch::asm!("invlpg [{}]", in(reg) vaddr) };
}

#[allow(static_mut_refs)]
pub fn init_memory(info: &MultibootInfo) {
    set_mmap_entries_in_used_pages(info);
    set_first_megabyte_to_used();
    set_available_memory(info);
    kernel_page_mappings_create();
    unset_identity_mapping();
    page_directory_fill_empty();
    enable_read_write_enforcement();
}

fn set_available_memory(info: &MultibootInfo) {
    printkln!("MEM UPPER {} kb", info.mem_upper);
    unsafe {
        #[allow(static_mut_refs)]
        #[allow(clippy::needless_range_loop)]
        for i in ((info.mem_upper * 1024) as usize / PAGE_SIZE)..USED_PAGES.len() {
            USED_PAGES[i] = Some(Access::Root);
        }
    }
}

fn set_first_megabyte_to_used() {
    #[allow(clippy::needless_range_loop)]
    for i in 0..(0xFFFFF / PAGE_SIZE) {
        unsafe { USED_PAGES[i] = Some(Access::Root) }
    }
}
fn set_mmap_entries_in_used_pages(info: &MultibootInfo) {
    let mut i = 0;

    loop {
        unsafe {
            let entry: *const MultibootMmapEntry = (info.mmap_addr + i) as *const MultibootMmapEntry;
            printkln!("addr: 0x{:09X} | len : 0x{:08X} | type : {:x}", (*entry).addr, (*entry).len, (*entry).ty);
            if (*entry).ty != 1 {
                for i in 0..((*entry).len as usize / PAGE_SIZE) {
                    let index = ((*entry).addr as usize / PAGE_SIZE) + i;
                    USED_PAGES[index] = Some(Access::Root);
                }
            }

            i += (*entry).size + 4;
            if i >= info.mmap_length {
                break;
            }
        }
    }
}
fn kernel_page_mappings_create() {
    let kernel_end = &raw const KERNEL_END as usize;
    let kernel_pages_needed = ((kernel_end + 1) - KERNEL_BASE) / PAGE_SIZE;

    for (i, _) in unsafe { USED_PAGES }.iter_mut().enumerate().take(kernel_pages_needed) {
        unsafe {
            USED_PAGES[i] = Some(Access::Root);
        }

        let dir_index = i / PAGE_TABLE_SIZE;
        let page_index = i % PAGE_TABLE_SIZE;
        let mut e = PageTableEntry::empty();
        e.set_address(i as u32);
        e.set_read_write(1);
        e.set_present(1);

        unsafe {
            KERNEL_PAGE_TABLES[768 + dir_index].0[page_index] = e;
        }
    }

    let mut kernel_page_entries_physical_address = &raw const KERNEL_PAGE_TABLES as usize;
    kernel_page_entries_physical_address -= KERNEL_BASE;

    for i in 0..=(kernel_pages_needed / PAGE_TABLE_SIZE) {
        let mut e = PageDirectoryEntry::empty();
        e.set_address((kernel_page_entries_physical_address / PAGE_SIZE) as u32 + i as u32 + 768);
        e.set_read_write(1);
        e.set_present(1);

        unsafe {
            KERNEL_PAGE_DIRECTORY_TABLE.0[768 + i] = e;
        }
    }

    for i in 0..=kernel_pages_needed {
        invalidate(KERNEL_BASE + i * PAGE_SIZE);
    }
}

fn page_directory_fill_empty() {
    let mut kernel_page_entries_physical_address = &raw const KERNEL_PAGE_TABLES as usize;
    kernel_page_entries_physical_address -= KERNEL_BASE;

    #[allow(static_mut_refs)]
    unsafe {
        for (i, entry) in KERNEL_PAGE_DIRECTORY_TABLE.0.iter_mut().enumerate() {
            let already_set = entry.address() != 0;
            if already_set {
                continue;
            }

            let mut e = PageDirectoryEntry::empty();
            e.set_address((kernel_page_entries_physical_address / PAGE_SIZE) as u32 + i as u32);
            e.set_read_write(1);
            e.set_present(1);

            *entry = e;
        }
    }
}

fn unset_identity_mapping() {
    unsafe { KERNEL_PAGE_DIRECTORY_TABLE.0[0] = PageDirectoryEntry::empty() }

    invalidate(0);
}

fn enable_read_write_enforcement() {
    let mut cr0: u32;
    unsafe {
        asm!("mov {}, cr0", out(reg) cr0);
    }

    cr0 |= 1 << 16;

    unsafe {
        asm!("mov cr0, {}", in(reg) cr0);
    }
}
