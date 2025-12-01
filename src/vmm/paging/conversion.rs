use crate::vmm::paging::{page_entries::PageTableEntry, state::KERNEL_PAGE_TABLES};

fn _page_get(_vaddr: usize) -> Option<*const PageTableEntry> {
    Some(unsafe { &KERNEL_PAGE_TABLES[0].0[0] })
}
