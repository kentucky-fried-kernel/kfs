use alloc::boxed::Box;

use crate::{
    arch::x86::vmm::{
        page::{PAGE_DIRECTORY_SIZE, PageDirectory, PageDirectoryEntry},
        state::PAGE_DIRECTORY_KERNEL,
    },
    boot::KERNEL_BASE,
};

/// The address space of a process, containing its page directory.
///
/// The upper-half kernel entries (768–1023) are copied from the kernel page
/// directory on creation so the process can still access the kernel after a
/// CR3 switch.
#[derive(Debug)]
pub struct AddressSpace {
    pub page_directory: Box<PageDirectory>,
}

impl AddressSpace {
    /// Creates a new address space, copying the kernel page directory entries
    /// (upper half, indices 768–1023) so the kernel remains accessible.
    ///
    /// # Panics
    /// Panics if the kernel page directory lock cannot be acquired.
    pub fn new() -> Self {
        let mut page_directory =
            Box::new(PageDirectory([PageDirectoryEntry::empty(); PAGE_DIRECTORY_SIZE]));

        let kernel_pd =
            PAGE_DIRECTORY_KERNEL.lock().expect("failed to lock PAGE_DIRECTORY_KERNEL");
        page_directory.0[768..].copy_from_slice(&kernel_pd.0[768..]);
        drop(kernel_pd);

        AddressSpace { page_directory }
    }

    /// Returns the physical address of the page directory, suitable for
    /// loading into CR3.
    ///
    /// The page directory lives at a kernel virtual address; subtracting
    /// `KERNEL_BASE` gives the corresponding physical address that the MMU
    /// expects in CR3.
    pub fn cr3(&self) -> *mut PageDirectory {
        let vaddr = self.page_directory.as_ref() as *const PageDirectory as usize;
        (vaddr - KERNEL_BASE) as *mut PageDirectory
    }
}
