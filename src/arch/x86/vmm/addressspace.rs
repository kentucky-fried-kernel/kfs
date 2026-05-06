use alloc::{boxed::Box, collections::BTreeMap, vec::Vec};

use crate::{
    arch::x86::vmm::{
        PAGE_SIZE,
        init::load_page_directory,
        page::{PAGE_DIRECTORY_SIZE, PAGE_TABLE_SIZE, PageDirectory, PageDirectoryEntry, PageTable, PageTableEntry},
        state::{PAGE_ALLOCATOR, PAGE_DIRECTORY_KERNEL},
    },
    boot::KERNEL_BASE,
};

pub struct Addressspace {
    page_directory: Box<PageDirectory>,
    page_tables: BTreeMap<u16, Box<PageTable>>,
}

impl Addressspace {
    pub fn new() -> Self {
        let mut pd = Box::new(PageDirectory::empty());

        let page_directory_kernel = PAGE_DIRECTORY_KERNEL.lock().expect("addressspace::new() | couldn't lock PAGE_ALLOCATOR");

        for i in 0..PAGE_DIRECTORY_SIZE {
            pd[i] = page_directory_kernel[i];
        }
        Self {
            page_directory: pd,
            page_tables: BTreeMap::new(),
        }
    }

    pub fn map(&mut self, vaddr: *const u8, paddr: *const u8, permissions: u8) {
        let vaddr = vaddr as usize;
        let paddr = paddr as usize;

        let pde_idx = vaddr >> 22;
        let pt_idx = (vaddr >> 12) & 0x3FF;
        assert!(pde_idx < 768, "user mapping in kernel half");

        let pt_paddr = {
            let (_pt, pt_paddr) = self.get_or_create(pde_idx as u16);
            pt_paddr
        };

        let mut pde = PageDirectoryEntry::empty();
        pde.set_address((pt_paddr >> 12) as u32);
        pde.set_user_supervisor(1);
        pde.set_read_write(1);
        pde.set_present(1);
        self.page_directory[pde_idx] = pde;

        let pt = self.page_tables.get_mut(&(pde_idx as u16)).expect("just created");
        let mut pte = PageTableEntry::empty();
        pte.set_address((paddr >> 12) as u32);
        pte.set_user_supervisor(1);
        pte.set_read_write(permissions);
        pte.set_present(1);
        pt[pt_idx] = pte;

        unsafe {
            core::arch::asm!("invlpg [{}]", in(reg) vaddr, options(nostack, preserves_flags));
        }
    }

    pub fn unmap(&mut self, vaddr: *const u8) {
        let vaddr = vaddr as usize;
        let pde_idx = (vaddr >> 22) as u16;

        let Some(pt) = self.page_tables.get_mut(&pde_idx) else { return };
        let pt_idx = (vaddr >> 12) & 0x3FF;
        pt[pt_idx] = PageTableEntry::empty();

        if pt.iter().all(|pte| pte.present() == 0) {
            self.page_directory[pde_idx as usize] = PageDirectoryEntry::empty();
            self.page_tables.remove(&pde_idx);
        }
    }

    pub fn get_or_create(&mut self, pde_idx: u16) -> (&mut PageTable, usize) {
        let pt = self.page_tables.entry(pde_idx).or_insert_with(|| Box::new(PageTable::empty()));
        let paddr = (&raw const **pt as usize) - KERNEL_BASE;
        (pt, paddr)
    }

    pub fn remove(&mut self, pde_idx: u16) -> Option<Box<PageTable>> {
        self.page_tables.remove(&pde_idx)
    }

    fn cr3(&self) -> *const PageDirectory {
        unsafe { ((&*self.page_directory as *const _ as usize - KERNEL_BASE) as *const PageDirectory) }
    }

    pub fn load(&self) {
        let addr = self.cr3();
        assert!(addr as usize % PAGE_SIZE == 0);

        unsafe {
            core::arch::asm!("mov cr3, {}", in(reg) addr, options(nostack, preserves_flags));
        }
    }
}
