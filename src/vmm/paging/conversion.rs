use crate::vmm::paging::{page_entries::PageTableEntry, state::KERNEL_PAGE_TABLES};

fn page_get(vaddr: usize) -> Option<*const PageTableEntry> {
    return Some(unsafe { &KERNEL_PAGE_TABLES[0].0[0] });
}