use crate::vmm::paging::{
    Access, PAGE_SIZE, Permissions,
    page_entries::PageTableEntry,
    state::{KERNEL_PAGE_TABLES, USED_PAGES},
};

#[derive(Debug)]
pub enum MmapError {
    VaddrRangeAlreadyMapped,
    NotEnoughMemory,
    NotImplemented,
}

#[allow(static_mut_refs)]
pub fn mmap(vaddr: Option<usize>, size: usize, permissions: Permissions, access: Access) -> Result<usize, MmapError> {
    if vaddr.is_some() {
        unimplemented!();
    }

    let pages_needed = (PAGE_SIZE + size - 1) / PAGE_SIZE;

    unsafe {
        let page_physical_used_highest_index_root = match USED_PAGES.iter().enumerate().find(|(_i, p)| p.is_some_and(|p| p == Access::Root)) {
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

                let pages_virtual_free_iter = KERNEL_PAGE_TABLES.iter_mut().flat_map(|p| &mut p.0).enumerate();
                let pages_virtual_to_be_allocated = {
                    let mut res = None;
                    for (i, _p) in pages_virtual_free_iter {
                        let all_are_free = pages_needed
                            == KERNEL_PAGE_TABLES
                                .iter_mut()
                                .flat_map(|p| &mut p.0)
                                .skip(i)
                                .take(pages_needed)
                                .filter(|p| p.present() == 0)
                                .count();
                        if all_are_free {
                            res = Some(KERNEL_PAGE_TABLES.iter_mut().flat_map(|p| &mut p.0).enumerate().skip(i));
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

                let mut return_value = Err(MmapError::NotEnoughMemory);
                for ((physical_i, physical_page), (virtual_i, virtual_page)) in pages_to_be_allocated {
                    *physical_page = Some(access);

                    let mut e = PageTableEntry::empty();
                    e.set_address(physical_i as u32);
                    e.set_read_write(permissions as u8);
                    e.set_present(1);
                    *virtual_page = e;

                    if return_value.is_err() {
                        return_value = Ok(virtual_i * PAGE_SIZE);
                    }
                }

                return_value
            }
        }
    }
}
