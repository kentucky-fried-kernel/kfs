use core::{
    arch::asm,
    u32, usize,
};

use crate::{
    _kernel_end,
    arch::x86::vmm::{
        allocators::kmalloc,
        page::{PAGE_SIZE, PageDirectory, PageDirectoryEntry, PageTableEntry},
        page_allocator::ORDERS,
        state::{PAGE_ALLOCATOR, PAGE_DIRECTORY_KERNEL, PAGE_TABLES_KERNEL},
    },
    boot::{KERNEL_BASE, MultibootInfo, MultibootMmapEntry},
};

pub fn init(info: &MultibootInfo) -> Result<(), ()> {
    mark_reserved_regions(info);
    mark_above_available(info);
    map_kernel()?;
    enable_read_write_enforcement();
    kmalloc::init();
    Ok(())
}

fn mark_reserved_regions(info: &MultibootInfo) {
    let mut offset: u32 = 0;

    let mut allocator = PAGE_ALLOCATOR.lock().expect("failed to aquire lock on PAGE_ALLOCATOR");
    while offset < info.mmap_length {
        let entry = unsafe { *((info.mmap_addr + offset) as *const MultibootMmapEntry) };

        let high_memory_used = entry.addr >= 0x1_0000_0000;
        if high_memory_used {
            panic!("This kernel doesn't support high memory that is mapped over 4GB - please install less than 3.5GB of ram");
        }

        if entry.ty != 1 {
            let base = (entry.addr as usize) & !(PAGE_SIZE - 1);
            let end = (entry.addr as usize + entry.len as usize + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
            let mut addr = base;

            if addr < 0x100000 {
                // The lower memory will be allocated in
                // [map_kernel] where the alloc is not allowed to fail.
                offset += entry.size + 4;
                continue;
            }
            while addr < end {
                allocator
                    .alloc_at(addr as *mut u8, PAGE_SIZE)
                    .expect("failed to remove reserved memory regions from info mmap entries");
                addr += PAGE_SIZE;
            }
        }
        offset += entry.size + 4;
    }
}

fn pow2(power: usize) -> usize {
    2u32.pow(power as u32) as usize
}

fn mark_above_available(info: &MultibootInfo) {
    let mut allocator = PAGE_ALLOCATOR.lock().expect("failed to acquire lock");
    let mut offset: u32 = 0;
    let mut highest_available_end: u64 = 0;

    while offset < info.mmap_length {
        let entry = unsafe { *((info.mmap_addr + offset) as *const MultibootMmapEntry) };

        if entry.ty == 1 {
            let end = entry.addr.saturating_add(entry.len);

            if end > 0x1_0000_0000 {
                panic!("This kernel doesn't support high memory that is mapped over 4GB - please install less than 3.5GB of ram");
            }

            if end > highest_available_end {
                highest_available_end = end;
            }
        }

        offset += entry.size + 4;
    }

    const ADDR_SPACE_END: u64 = 1u64 << 32;

    let mut addr: u64 = highest_available_end & !((PAGE_SIZE as u64) - 1);

    while addr < ADDR_SPACE_END {
        // Find the largest power-of-2 block that:
        // 1. is naturally aligned at addr
        // 2. doesn't go past ADDR_SPACE_END
        let mut order = ORDERS - 2;
        loop {
            let block_size = (pow2(order) as u64) * (PAGE_SIZE as u64);
            let aligned = addr % block_size == 0;
            let fits = block_size <= ADDR_SPACE_END - addr;
            if aligned && fits {
                break;
            }
            if order == 0 {
                break;
            }
            order -= 1;
        }

        let block_size = (pow2(order) as u64) * (PAGE_SIZE as u64);
        let _ = allocator.alloc_at(addr as *mut u8, block_size as usize);
        addr += block_size;
    }
}

fn enable_read_write_enforcement() {
    let mut cr0: u32;
    // Safety:
    // We just read so nothing bad can happen yet
    unsafe {
        asm!("mov {}, cr0", out(reg) cr0);
    }

    cr0 |= 1 << 16;

    // Safety:
    // here we make sure that we set the read
    // write protection bit and write it back
    unsafe {
        asm!("mov cr0, {}", in(reg) cr0);
    }
}

fn map_kernel() -> Result<(), ()> {
    let kernel_end: usize = &raw const _kernel_end as usize;
    let size = kernel_end - KERNEL_BASE;
    mmap_init(KERNEL_BASE as *mut u8, Some(0 as *mut u8), size);

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

pub(super) fn mmap_init(vaddr: *mut u8, paddr: Option<*mut u8>, size: usize) -> Result<(), ()> {
    assert!(vaddr as usize % PAGE_SIZE == 0);
    if let Some(paddr) = paddr {
        assert!(paddr as usize % PAGE_SIZE == 0);
    }

    let mut allocator = PAGE_ALLOCATOR.lock().expect("failed to aquire mutex on PAGE_ALLOCATOR");

    let paddr = match paddr {
        Some(paddr) => allocator.alloc_at(paddr, size).expect("couldn't find enough memory for kernel on boot"),
        None => allocator.alloc(size).expect("couldn't find enough memory for kernel on boot"),
    };

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
