use crate::boot::{INITIAL_PAGE_DIR, KERNEL_BASE};

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

fn invalidate(vaddr: usize) {
    unsafe { core::arch::asm!("invlpg [{}]", in(reg) vaddr) };
}

#[allow(static_mut_refs)]
pub fn init_memory(_mem_high: usize, _physical_alloc_start: usize) {
    // We do not need the identity mapped kernel anymore, so we can remove
    // its PD entry.
    unsafe { INITIAL_PAGE_DIR[0] = 0 };
    invalidate(0);

    let page_dir_phys = unsafe { (&INITIAL_PAGE_DIR as *const _ as usize) - KERNEL_BASE };

    let page_dir_entry: u32 = PageDirectoryEntry::new(page_dir_phys as u32 | 1 | 2).into();
    // Recursive mapping (maps the page directory itself into virtual memory)
    unsafe { INITIAL_PAGE_DIR[1023] = page_dir_entry as usize };
    invalidate(0xFFFFF000);
}
