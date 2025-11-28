use crate::{
    boot::KERNEL_BASE,
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

#[allow(static_mut_refs)]
fn pages_virtual_iter(access: Access) -> impl Iterator<Item = (usize, &'static mut PageTableEntry)> {
    unsafe {
        let pages_virtual_all = KERNEL_PAGE_TABLES.iter_mut().flat_map(|p| &mut p.0);

        // NOTE: have to .skip(0) here so that the types are the same
        #[allow(clippy::iter_skip_zero)]
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
    let _lets_see = pages_physical_iter();

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
    if vaddr.is_some() {
        unimplemented!();
    }

    let pages_needed = (PAGE_SIZE + size - 1) / PAGE_SIZE;

    let pages_virtual = pages_virtual_free_iter(pages_needed, access)?;

    let pages_physical = pages_physical_free_iter(pages_needed, mode)?;

    let pages = pages_physical.zip(pages_virtual);

    let mut first_page_addr = Err(MmapError::NotImplemented);
    for ((physical_i, physical_page), (virtual_i, virtual_page)) in pages {
        if first_page_addr.is_err() {
            first_page_addr = Ok(virtual_i * PAGE_SIZE);
        }

        *physical_page = Some(access);

        let mut e = PageTableEntry::empty();
        e.set_address(physical_i as u32);
        e.set_read_write(permissions as u8);
        e.set_present(1);

        *virtual_page = e;
    }

    first_page_addr
}
