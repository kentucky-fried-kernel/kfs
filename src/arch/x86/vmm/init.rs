use core::{
    iter::{self},
    u32,
};

use crate::{
    _kernel_end,
    arch::x86::vmm::{
        page::{PAGE_SIZE, PageDirectory, PageDirectoryEntry, PageTableEntry},
        page_allocator::{ORDERS, PageAllocator},
        state::{PAGE_ALLOCATOR, PAGE_DIRECTORY_KERNEL, PAGE_TABLES_KERNEL},
    },
    boot::KERNEL_BASE,
    serial, serial_println,
};

pub fn init() -> Result<(), ()> {
    let kernel_end: usize = &raw const _kernel_end as usize;
    let size = kernel_end - KERNEL_BASE;
    mmap_init(KERNEL_BASE as *mut u8, 0 as *mut u8, size);

    let page_directory_paddr = &PAGE_DIRECTORY_KERNEL as *const _ as usize - KERNEL_BASE;
    let page_directory_paddr = page_directory_paddr as *mut PageDirectory;

    // Safety:
    // We make sure that the [PageDirectory] is filled and the
    // pointer is valid because we cast it from the global above.
    unsafe {
        load_page_directory(page_directory_paddr);
    }
    Ok(())
}

pub unsafe fn load_page_directory(addr: *mut PageDirectory) {
    assert!(addr as usize % PAGE_SIZE == 0);
    // Safety:
    // The caller has to make sure that the address is a filled
    // [PageDirectory] and is pagealligned
    unsafe {
        core::arch::asm!("mov cr3, {}", in(reg) addr, options(nostack, preserves_flags));
    }
}

fn mmap_init(vaddr: *mut u8, paddr: *mut u8, size: usize) -> Result<(), ()> {
    assert!(vaddr as usize % PAGE_SIZE == 0);
    assert!(paddr as usize % PAGE_SIZE == 0);

    let mut allocator = PAGE_ALLOCATOR.lock().expect("failed to aquire mutex on PAGE_ALLOCATOR");

    let paddr = allocator.alloc_at(paddr, size).ok_or(())?;
    drop(allocator);

    map_to(vaddr, paddr, bytes_to_pages(size));
    Ok(())
}

fn map_to(vaddr: *mut u8, paddr: *mut u8, pages: usize) {
    assert!(vaddr as usize % PAGE_SIZE == 0);
    assert!(paddr as usize % PAGE_SIZE == 0);
    assert!(pages != 0);

    let mut page_directory = PAGE_DIRECTORY_KERNEL.lock().expect("failed to aquire mutex on PAGE_DIRECTORY_KERNEL");
    let mut page_tables = PAGE_TABLES_KERNEL.lock().expect("failed to aquire mutex on PAGE_TABLES_KERNEL");

    for i in 0..pages {
        let vaddr: usize = vaddr as usize + i * PAGE_SIZE;
        let paddr: usize = paddr as usize + i * PAGE_SIZE;

        let page_directory_index = vaddr >> 22;
        const TABLE_KERNEL_PAGE_TABLE_OFFSET: usize = 768;
        let page_table_paddr = &mut page_tables[page_directory_index - TABLE_KERNEL_PAGE_TABLE_OFFSET] as *const _ as usize - KERNEL_BASE;

        let page_table_index = (vaddr >> 12) & 0b11_11_11_11_11;
        let page_directory_entry: &mut PageDirectoryEntry = &mut page_directory[page_directory_index];
        let page_table_entry: &mut PageTableEntry = &mut page_tables[page_directory_index - TABLE_KERNEL_PAGE_TABLE_OFFSET][page_table_index];

        let mut pde = PageDirectoryEntry::empty();
        pde.set_address((page_table_paddr >> 12) as u32);
        pde.set_read_write(1);
        pde.set_present(1);
        *page_directory_entry = pde;

        let mut pte = PageTableEntry::empty();
        pte.set_address((paddr >> 12) as u32);
        pte.set_read_write(1);
        pte.set_present(1);
        *page_table_entry = pte;
    }
}

fn bytes_to_pages(size: usize) -> usize {
    (size + PAGE_SIZE - 1) / PAGE_SIZE
}
