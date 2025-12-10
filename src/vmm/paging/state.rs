use crate::vmm::{
    MEMORY_MAX,
    paging::{
        Access,
        page_entries::{PageDirectory, PageDirectoryEntry, PageTable, PageTableEntry},
    },
};

pub const USED_PAGES_SIZE: usize = (MEMORY_MAX / super::PAGE_SIZE as u64) as usize;
#[used]
#[unsafe(no_mangle)]
#[allow(clippy::identity_op)]
#[unsafe(link_section = ".data")]
pub static mut USED_PAGES: [Option<Access>; USED_PAGES_SIZE] = [None; USED_PAGES_SIZE];

pub const PAGE_TABLE_SIZE: usize = 1024;
#[used]
#[unsafe(no_mangle)]
#[allow(clippy::identity_op)]
#[unsafe(link_section = ".data")]
pub static mut KERNEL_PAGE_TABLES: [PageTable; PAGE_TABLE_SIZE] = [PageTable([PageTableEntry::empty(); PAGE_TABLE_SIZE]); PAGE_TABLE_SIZE];

pub const KERNEL_PAGE_DIRECTORY_TABLE_SIZE: usize = 1024;
#[used]
#[unsafe(no_mangle)]
#[allow(clippy::identity_op)]
#[unsafe(link_section = ".data")]
pub static mut KERNEL_PAGE_DIRECTORY_TABLE: PageDirectory = {
    let mut dir: [PageDirectoryEntry; KERNEL_PAGE_DIRECTORY_TABLE_SIZE] = [PageDirectoryEntry::from(0); KERNEL_PAGE_DIRECTORY_TABLE_SIZE];

    dir[0] = PageDirectoryEntry::from((0 << 22) | 0b1000_0011);

    // Sets mappings temporary so that the kernel is mapped to the upper half of the
    // vm space
    dir[768] = PageDirectoryEntry::from((0 << 22) | 0b1000_0011);
    dir[769] = PageDirectoryEntry::from((1 << 22) | 0b1000_0011);
    dir[770] = PageDirectoryEntry::from((2 << 22) | 0b1000_0011);
    dir[771] = PageDirectoryEntry::from((3 << 22) | 0b1000_0011);

    PageDirectory(dir)
};
