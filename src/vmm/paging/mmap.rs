use crate::{
    boot::KERNEL_BASE,
    vmm::{
        MEMORY_MAX,
        paging::{
            Access, PAGE_SIZE, Permissions,
            init::KERNEL_END,
            page_entries::PageTableEntry,
            state::{self, KERNEL_PAGE_TABLES, USED_PAGES},
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
                .take((MEMORY_MAX - (KERNEL_BASE as u64 / PAGE_SIZE as u64)) as usize),
        }
    }
}

fn pages_virtual_free_iter(pages_needed: usize, access: Access) -> Result<impl Iterator<Item = (usize, &'static mut PageTableEntry)>, MmapError> {
    let mut i = 0;
    loop {
        if i >= pages_virtual_iter(access).count() {
            break;
        }
        let pages_virtual = pages_virtual_iter(access).skip(i).take(pages_needed).filter(|(_, p)| p.present() == 0);
        if pages_virtual.count() == pages_needed {
            return Ok(pages_virtual_iter(access).skip(i).take(pages_needed));
        } else {
            match pages_virtual_iter(access).skip(i).take(pages_needed).filter(|(_, p)| p.present() == 1).last() {
                Some((x, _)) => i = x + 1 - KERNEL_BASE / PAGE_SIZE,
                None => {
                    return Err(MmapError::VaddrRangeNotAvailable);
                }
            }
        }
    }

    Err(MmapError::VaddrRangeNotAvailable)
}

#[allow(static_mut_refs)]
fn pages_physical_iter() -> impl Iterator<Item = (usize, &'static mut Option<Access>)> {
    unsafe { USED_PAGES.iter_mut().enumerate() }
}

fn pages_physical_free_iter(pages_needed: usize, mode: &Mode) -> Result<impl Iterator<Item = (usize, &'static mut Option<Access>)>, MmapError> {
    let _lets_see = pages_physical_iter();

    let kernel_end_phys = unsafe { KERNEL_END } as *const u8 as usize - KERNEL_BASE;

    let i = match mode {
        Mode::Continous => {
            if pages_physical_iter()
                .skip(kernel_end_phys / PAGE_SIZE)
                .filter(|(_, p)| p.is_none())
                .take(pages_needed)
                .count()
                == pages_needed
            {
                Ok(kernel_end_phys / PAGE_SIZE)
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

#[derive(Debug)]
pub enum VirtToPhysError {
    PageNotPresent,
    PageDirectoryNotPresent,
}

/// # Errors
/// This function will return an error if `vaddr` does not point to an allocated
/// phyisical address.
#[allow(static_mut_refs)]
pub fn virt_to_phys(vaddr: usize) -> Result<usize, VirtToPhysError> {
    let page_directory_index = vaddr >> 22;
    let page_table_index = (vaddr >> 12) & 0x3FF;
    let offset = vaddr & 0xFFF;

    unsafe {
        let page_directory_entry = &state::KERNEL_PAGE_DIRECTORY_TABLE.0[page_directory_index];

        if page_directory_entry.present() == 0 {
            return Err(VirtToPhysError::PageDirectoryNotPresent);
        }

        if page_directory_entry.ps() == 1 {
            let phys_base = (page_directory_entry.address() as usize) << 12;
            let offset_4mb = vaddr & 0x3F_FF_FF;
            return Ok(phys_base + offset_4mb);
        }

        let page_table_entry = &KERNEL_PAGE_TABLES[page_directory_index].0[page_table_index];

        if page_table_entry.present() == 0 {
            return Err(VirtToPhysError::PageNotPresent);
        }

        let phys_page = (page_table_entry.address() as usize) << 12;
        Ok(phys_page + offset)
    }
}

/// # Errors
/// todo @fbruggem
#[allow(static_mut_refs)]
pub fn mmap(vaddr: Option<usize>, size: usize, permissions: Permissions, access: Access, mode: &Mode) -> Result<usize, MmapError> {
    if vaddr.is_some() {
        unimplemented!();
    }

    let pages_needed = (PAGE_SIZE + size - 1) / PAGE_SIZE;

    let pages_physical = pages_physical_free_iter(pages_needed, &mode)?;
    if pages_physical.count() * PAGE_SIZE < size {
        return Err(MmapError::NotEnoughMemory);
    }
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

pub enum MunmapError {
    SizeIsZero,
}

/// # Errors
/// todo @fbruggem
///
/// # Panics
/// todo @fbruggem
#[allow(static_mut_refs)]
pub fn munmap(vaddr: usize, size: usize) -> Result<(), MunmapError> {
    let size = (size + (size % PAGE_SIZE)) / PAGE_SIZE;
    if size == 0 {
        return Err(MunmapError::SizeIsZero);
    }
    for i in 0..size {
        let vaddr = vaddr + i * PAGE_SIZE;
        let page_directory_index = vaddr >> 22;
        let page_table_index = (vaddr << 10) >> 22;

        unsafe {
            let page_directory_entry = &mut state::KERNEL_PAGE_DIRECTORY_TABLE.0[page_directory_index];

            assert_ne!(page_directory_entry.ps(), 1, "page directories with size 4mb are not supported");

            let page_table_entry = &mut KERNEL_PAGE_TABLES[page_directory_index].0[page_table_index];
            if page_table_entry.present() == 1 {
                USED_PAGES[page_table_entry.address() as usize] = None;
            }
            *page_table_entry = PageTableEntry::empty();
        }
    }
    Ok(())
}
