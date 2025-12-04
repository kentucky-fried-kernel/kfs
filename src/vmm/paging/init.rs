use core::arch::asm;

use crate::{
    boot::KERNEL_BASE,
    serial_println,
    vmm::paging::{
        Access, PAGE_SIZE,
        page_entries::{PageDirectoryEntry, PageTableEntry},
        state::{KERNEL_PAGE_DIRECTORY_TABLE, KERNEL_PAGE_TABLES, PAGE_TABLE_SIZE, USED_PAGES},
    },
};

unsafe extern "C" {
    #[link_name = "text_start"]
    static TEXT_START: u8;
    #[link_name = "data_start"]
    static DATA_START: u8;
    #[link_name = "_bss_start"]
    static BSS_START: u8;
    #[link_name = "_kernel_end"]
    pub static KERNEL_END: u8;
}

fn invalidate(vaddr: usize) {
    unsafe { core::arch::asm!("invlpg [{}]", in(reg) vaddr) };
}

#[allow(static_mut_refs)]
pub fn init_memory(_mem_high: usize, _physical_alloc_start: usize) {
    serial_println!("VIRT text_start: 0x{:x}", unsafe { (&TEXT_START) as *const u8 as usize });
    serial_println!(
        "PHYS text_start: 0x{:x}",
        super::mmap::virt_to_phys(unsafe { (&TEXT_START) as *const u8 as usize }).expect("This page should definitely be mapped")
    );
    serial_println!("VIRT data_start: 0x{:x}", unsafe { (&DATA_START) as *const u8 as usize });
    serial_println!(
        "PHYS data_start: 0x{:x}",
        super::mmap::virt_to_phys(unsafe { (&DATA_START) as *const u8 as usize }).expect("This page should definitely be mapped")
    );
    serial_println!("VIRT _bss_start: 0x{:x}", unsafe { (&BSS_START) as *const u8 as usize });
    serial_println!(
        "PHYS _bss_start: 0x{:x}",
        super::mmap::virt_to_phys(unsafe { (&BSS_START) as *const u8 as usize }).expect("This page should definitely be mapped")
    );
    serial_println!("VIRT _kernel_end: 0x{:x}", unsafe { (&KERNEL_END) as *const u8 as usize });
    serial_println!(
        "PHYS _kernel_end: 0x{:x}",
        super::mmap::virt_to_phys(unsafe { (&KERNEL_END) as *const u8 as usize }).expect("This page should definitely be mapped")
    );
    kernel_page_mappings_create();
    unset_identity_mapping();
    page_directory_fill_empty();
    enable_read_write_enforcement();
}

fn kernel_page_mappings_create() {
    let kernel_end = unsafe { &KERNEL_END as *const _ } as usize;
    let kernel_pages_needed = ((kernel_end + 1) - KERNEL_BASE) / PAGE_SIZE;

    for (i, item) in unsafe { USED_PAGES }.iter_mut().enumerate().take(kernel_pages_needed) {
        *item = Some(Access::Root);

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
