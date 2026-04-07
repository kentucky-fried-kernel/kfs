use crate::arch::x86::vmm::page::{PAGE_DIRECTORY_SIZE, PAGE_TABLE_SIZE, PageAligned, PageDirectory, PageDirectoryEntry, PageTable, PageTableEntry};

#[used]
#[unsafe(no_mangle)]
#[allow(clippy::identity_op)]
#[unsafe(link_section = ".data")]
/// Temporary Page Directory making it possible to execute in Kernel Space
/// (upper half/higher half) without the whole memory being setup
/// No page tables are needed because the whole 4mb are being mapped
/// through the page directory
/// This is also used to bootstrap the page_allocators until the first process
/// has its own {PageDirectory}
pub(super) static mut PAGE_DIRECTORY_KERNEL: PageAligned<PageDirectory> = {
    let mut dir: [PageDirectoryEntry; PAGE_DIRECTORY_SIZE] = [PageDirectoryEntry::from(0); PAGE_DIRECTORY_SIZE];

    dir[0] = PageDirectoryEntry::from((0 << 22) | 0b1000_0011);

    dir[768] = PageDirectoryEntry::from((0 << 22) | 0b1000_0011);
    dir[769] = PageDirectoryEntry::from((1 << 22) | 0b1000_0011);
    dir[770] = PageDirectoryEntry::from((2 << 22) | 0b1000_0011);
    dir[771] = PageDirectoryEntry::from((3 << 22) | 0b1000_0011);
    dir[772] = PageDirectoryEntry::from((4 << 22) | 0b1000_0011);
    dir[773] = PageDirectoryEntry::from((5 << 22) | 0b1000_0011);
    dir[774] = PageDirectoryEntry::from((6 << 22) | 0b1000_0011);
    dir[775] = PageDirectoryEntry::from((7 << 22) | 0b1000_0011);
    dir[776] = PageDirectoryEntry::from((8 << 22) | 0b1000_0011);

    PageAligned(dir)
};

pub(super) const PAGE_TABLES_KERNEL_SIZE: usize = PAGE_DIRECTORY_SIZE / 4;
pub(super) static mut PAGE_TABLES_KERNEL: [PageTable; PAGE_TABLES_KERNEL_SIZE] = [[PageTableEntry::empty(); PAGE_TABLE_SIZE]; PAGE_TABLES_KERNEL_SIZE];
