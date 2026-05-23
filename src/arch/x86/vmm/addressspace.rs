use alloc::{boxed::Box, collections::BTreeMap};

use crate::{
    arch::x86::vmm::{
        PAGE_SIZE,
        page::{PAGE_DIRECTORY_SIZE, PAGE_TABLE_SIZE, PageDirectory, PageDirectoryEntry, PageTable, PageTableEntry},
        state::{PAGE_ALLOCATOR, PAGE_DIRECTORY_KERNEL},
    },
    binary::Permissions,
    boot::KERNEL_BASE,
};

pub struct Addressspace {
    page_directory: Box<PageDirectory>,
    page_tables: BTreeMap<u16, Box<PageTable>>,
}

impl Default for Addressspace {
    fn default() -> Self {
        Self::new()
    }
}

impl Addressspace {
    pub fn new() -> Self {
        let mut pd = Box::new(PageDirectory::empty());

        #[allow(clippy::missing_panics_doc)]
        let page_directory_kernel = PAGE_DIRECTORY_KERNEL.lock().expect("addressspace::new() | couldn't lock PAGE_ALLOCATOR");

        for i in 0..PAGE_DIRECTORY_SIZE {
            pd[i] = page_directory_kernel[i];
        }
        Self {
            page_directory: pd,
            page_tables: BTreeMap::new(),
        }
    }

    /// # Panics
    /// This panics if the vaddr is ove `KERNEL_BASE`
    pub fn map(&mut self, vaddr: *const u8, paddr: *const u8, permissions: Permissions) {
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
        pte.set_read_write(permissions as u8);
        pte.set_present(1);
        pt[pt_idx] = pte;

        // SAFETY:
        // This is safe because we just put this vaddr into the page table.
        unsafe {
            core::arch::asm!("invlpg [{}]", in(reg) vaddr, options(nostack, preserves_flags));
        }
    }
    pub fn fork(&mut self) -> Self {
        let mut child = Addressspace::new();
        const SCRATCH: usize = 0xBFFF_F000;

        for (&pde_idx, parent_pt) in &self.page_tables {
            if pde_idx >= 768 {
                continue;
            }

            for pt_idx in 0..PAGE_TABLE_SIZE {
                let parent_pte = parent_pt[pt_idx];
                if parent_pte.present() == 0 {
                    continue;
                }

                let vaddr = ((pde_idx as usize) << 22) | (pt_idx << 12);
                let _parent_paddr = (parent_pte.address() as usize) << 12;

                #[allow(clippy::missing_panics_doc)]

                crate::serial_println!("hello");
                let new_paddr = PAGE_ALLOCATOR
                    .lock()
                    .expect("fork | couldn't lock PAGE_ALLOCATOR")
                    .alloc(PAGE_SIZE)
                    .expect("fork | out of memory");
                crate::serial_println!("hello1");

                // SAFETY:
                // We use this so that we can go around the borrow checker because we loop over the
                // page and only map the pages to `SCRATCH`, to copy over the memory from the
                // parent to the child.
                unsafe {
                    let parent_mut = &mut *(self as *const Self as *mut Self);
                    parent_mut.map(SCRATCH as *const u8, new_paddr, Permissions::ReadWrite);
                }

                let src = vaddr as *const u8;
                let dst = SCRATCH as *mut u8;

                // TODO:
                // make it so that a mmap call to the SCRATCH address fails.
                // SAFETY:
                // This is save because we just mapped dst and src is mapped throug a previous map
                unsafe {
                    core::ptr::copy_nonoverlapping(src, dst, PAGE_SIZE);
                }

                // SAFETY:
                // We use this so that we can go around the borrow checker because we loop over the
                // page and only umap the pages to `SCRATCH`,  after we copied the memory from the
                // parent
                unsafe {
                    let parent_mut = &mut *(self as *const Self as *mut Self);
                    parent_mut.unmap(SCRATCH as *const u8);
                }

                let perm = match parent_pte.read_write() {
                    0 => Permissions::Read,
                    _ => Permissions::ReadWrite,
                };
                child.map(vaddr as *const u8, new_paddr, perm);
            }
        }

        child
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
        (&*self.page_directory as *const _ as usize - KERNEL_BASE) as *const PageDirectory
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn load(&self) {
        let addr = self.cr3();
        assert!(addr as usize % PAGE_SIZE == 0);

        // SAFETY:
        // This is safe because we make sure on new() that this is a page aligned address that is
        // filled. And we also check it again one line up.
        unsafe {
            core::arch::asm!("mov cr3, {}", in(reg) addr, options(nostack, preserves_flags));
        }
    }
}
impl Drop for Addressspace {
    fn drop(&mut self) {
        for pt in &self.page_tables {
            for page in pt.1.iter().filter(|pte| pte.present() == 1) {
                let paddr = page.address() << 12;
                let mut alloc = PAGE_ALLOCATOR.lock().expect("could not lock PAGE_ALLOCATOR");
                alloc.dealloc(paddr as *mut u8, PAGE_SIZE);
            }
        }
    }
}
