use crate::{
    boot::KERNEL_BASE,
    printkln,
    vmm::{
        MEMORY_MAX,
        paging::{
            Access, PAGE_SIZE, Permissions,
            page_entries::PageTableEntry,
            state::{KERNEL_PAGE_TABLES, USED_PAGES},
        },
    },
};

#[derive(Debug)]
pub enum MmapError {
    VaddrRangeNotAvailable,
    NotEnoughMemory,
    NotImplemented,
}

#[derive(PartialEq)]
pub enum Mode {
    Continous,
    Scattered,
}

// fn free_pages_physical_iter(iter: PhysicalIterator, pages_needed: usize, mode: Mode) -> Result<PhysicalIterator, MmapError> {}

#[allow(static_mut_refs)]
fn pages_virtual_iter(access: Access) -> impl Iterator<Item = (usize, &'static mut PageTableEntry)> {
    unsafe {
        let pages_virtual_all = KERNEL_PAGE_TABLES.iter_mut().flat_map(|p| &mut p.0);

        // NOTE: have to .skip(0) here so that the types are the same
        match access {
            Access::User => pages_virtual_all.enumerate().skip(0).take(KERNEL_BASE / PAGE_SIZE),
            Access::Root => pages_virtual_all
                .enumerate()
                .skip(KERNEL_BASE / PAGE_SIZE)
                .take(MEMORY_MAX - (KERNEL_BASE / PAGE_SIZE)),
        }
    }
}

fn pages_virtual_free_iter(pages_needed: usize, access: Access) -> Result<impl Iterator<Item = (usize, &'static mut PageTableEntry)>, MmapError> {
    let pages_virtual = pages_virtual_iter(access);

    for i in 0..pages_virtual.count() {
        let pages_virtual = pages_virtual_iter(access).skip(i).take(pages_needed).filter(|(_, p)| p.present() == 0);
        if pages_virtual.count() == pages_needed {
            return Ok(pages_virtual_iter(access).skip(i).take(pages_needed));
        }
    }

    Err(MmapError::VaddrRangeNotAvailable)
}

#[allow(static_mut_refs)]
fn pages_physical_iter() -> impl Iterator<Item = (usize, &'static mut Option<Access>)> {
    unsafe { USED_PAGES.iter_mut().enumerate() }
}

fn pages_physical_free_iter(pages_needed: usize, mode: Mode) -> Result<impl Iterator<Item = (usize, &'static mut Option<Access>)>, MmapError> {
    let lets_see = pages_physical_iter();

    let i = match mode {
        Mode::Continous => {
            if pages_physical_iter().filter(|(_, p)| p.is_none()).take(pages_needed).count() == pages_needed {
                Ok(0)
            } else {
                Err(MmapError::NotEnoughMemory)
            }
        }
        Mode::Scattered => {
            let mut res = Err(MmapError::NotEnoughMemory);
            for i in 0..pages_physical_iter().count() {
                let pages_physical = pages_physical_iter().skip(i).take(pages_needed).filter(|(_, p)| p.is_none());
                if pages_physical.count() == pages_needed {
                    res = Ok(i);
                }
            }
            res
        }
    }?;

    Ok(pages_physical_iter().skip(i).filter(|(_, p)| p.is_none()).take(pages_needed))
}

#[allow(static_mut_refs)]
pub fn mmap(vaddr: Option<usize>, size: usize, permissions: Permissions, access: Access, mode: Mode) -> Result<usize, MmapError> {
    if let Some(_) = vaddr {
        unimplemented!();
    }

    let pages_needed = (PAGE_SIZE + size - 1) / PAGE_SIZE;

    let pages_virtual = pages_virtual_free_iter(pages_needed, access)?;

    let pages_physical = pages_physical_free_iter(pages_needed, mode)?;

    let pages = pages_physical.zip(pages_virtual);

    let mut first_page_addr = Err(MmapError::NotImplemented);
    for ((physical_i, physical_page), (virtual_i, virtual_page)) in pages {
        if let Err(_) = first_page_addr {
            first_page_addr = Ok(virtual_i * PAGE_SIZE);
        }

        printkln!("physical 0x{}", physical_i);
        printkln!("virtual 0x{:x}", virtual_i);

        *physical_page = Some(access);

        let mut e = PageTableEntry::empty();
        e.set_address(physical_i as u32);
        e.set_read_write(permissions as u8);
        e.set_present(1);

        *virtual_page = e;
    }

    first_page_addr
    // unsafe {
    //     let page_physical_used_lowest_index_user = match USED_PAGES.iter().rev().enumerate().find(|(i, p)| p.is_some_and(|p| p == Access::User)) {
    //         Some((i, _)) => i,
    //         None => u32::MAX as usize / PAGE_SIZE,
    //     };
    //     let page_physical_used_highest_index_root = match USED_PAGES.iter().enumerate().find(|(i, p)| p.is_some_and(|p| p == Access::Root)) {
    //         Some((i, _)) => i,
    //         None => 0,
    //     };

    //     match access {
    //         Access::Root => {
    //             unimplemented!()
    //         }
    //         Access::User => {
    //             let free_pages_physical_iter = USED_PAGES
    //                 .iter()
    //                 .enumerate()
    //                 .skip(page_physical_used_highest_index_root)
    //                 .rev()
    //                 // .take(USED_PAGES.len() - page_physical_used_highest_index_root)
    //                 .filter(|(_, b)| b.is_none())
    //                 .take(pages_needed);

    //             let free_pages = free_pages_physical_iter.count();
    //             if free_pages < pages_needed {
    //                 return Err(MmapError::NotImplemented);
    //             }

    //             let free_pages_physical_iter = USED_PAGES
    //                 .iter_mut()
    //                 .enumerate()
    //                 .rev()
    //                 .take(USED_PAGES.len() - page_physical_used_highest_index_root)
    //                 .filter(|(_, b)| b.is_none())
    //                 .take(pages_needed);
    //             let pages_physical_to_be_allocated = free_pages_physical_iter.take(pages_needed);

    //             let pages_virtual_free_iter = KERNEL_PAGE_TABLES.iter_mut().flat_map(|p| &mut p.0).enumerate();
    //             let pages_virtual_to_be_allocated = {
    //                 let mut res = None;
    //                 for (i, p) in pages_virtual_free_iter {
    //                     let all_are_free = pages_needed
    //                         == KERNEL_PAGE_TABLES
    //                             .iter_mut()
    //                             .flat_map(|p| &mut p.0)
    //                             .skip(i)
    //                             .take(pages_needed)
    //                             .filter(|p| p.present() == 0)
    //                             .count();
    //                     if all_are_free {
    //                         res = Some(KERNEL_PAGE_TABLES.iter_mut().flat_map(|p| &mut p.0).enumerate().skip(i));
    //                         break;
    //                     }
    //                 }
    //                 res
    //             };

    //             let pages_virtual_to_be_allocated = match pages_virtual_to_be_allocated {
    //                 Some(it) => it,
    //                 None => return Err(MmapError::VaddrRangeAlreadyMapped),
    //             };

    //             let pages_to_be_allocated = pages_physical_to_be_allocated.zip(pages_virtual_to_be_allocated);

    //             let mut return_value = Err(MmapError::NotEnoughMemory);
    //             for ((physical_i, physical_page), (virtual_i, virtual_page)) in pages_to_be_allocated {
    //                 printkln!("virt: {:x}", virtual_i);
    //                 printkln!("phys: {:x}", physical_i);
    //             }

    //             return return_value;
    //         }
    //     }
    // }
}
