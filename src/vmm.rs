use crate::{
    boot::{INITIAL_PAGE_DIR, KERNEL_BASE},
    printkln,
};

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
    printkln!("page_dir_phys: 0x{:x}", page_dir_phys);
    printkln!("page_dir_virt: 0x{:x}", unsafe { &INITIAL_PAGE_DIR as *const _ as usize });

    // Recursive mapping (maps the page directory itself into virtual memory)
    unsafe { INITIAL_PAGE_DIR[1023] = page_dir_phys | 1 | 2 };
    invalidate(0xFFFFF000);
}
