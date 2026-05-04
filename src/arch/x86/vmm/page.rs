pub const PAGE_SIZE: usize = 0x1000;

pub const PAGE_DIRECTORY_SIZE: usize = 1024;
#[repr(align(0x1000))]
#[derive(Debug, Clone)]
pub struct PageDirectory(pub [PageDirectoryEntry; PAGE_DIRECTORY_SIZE]);

impl core::ops::Deref for PageDirectory {
    type Target = [PageDirectoryEntry; PAGE_DIRECTORY_SIZE];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for PageDirectory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub const PAGE_TABLE_SIZE: usize = 1024;
#[repr(align(0x1000))]
#[derive(Copy, Clone, Debug)]
pub struct PageTable(pub [PageTableEntry; PAGE_TABLE_SIZE]);

impl core::ops::Deref for PageTable {
    type Target = [PageTableEntry; PAGE_TABLE_SIZE];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for PageTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[bitstruct::bitstruct]
pub struct PageDirectoryEntry {
    address: u20,
    available_4: u4,
    ps: u1,
    available_1: u1,
    accessed: u1,
    cache_disable: u1,
    write_through: u1,
    user_supervisor: u1,
    read_write: u1,
    present: u1,
}

impl const From<usize> for PageDirectoryEntry {
    fn from(value: usize) -> Self {
        Self(value as u32)
    }
}

impl const From<PageDirectoryEntry> for usize {
    fn from(value: PageDirectoryEntry) -> Self {
        value.0 as Self
    }
}

impl PageDirectoryEntry {
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }
}

#[bitstruct::bitstruct]
pub struct PageTableEntry {
    address: u20,
    available: u3,
    global: u1,
    page_attribute_table: u1,
    dirty: u1,
    accessed: u1,
    cache_disable: u1,
    write_through: u1,
    user_supervisor: u1,
    read_write: u1,
    present: u1,
}

impl const From<usize> for PageTableEntry {
    fn from(value: usize) -> Self {
        Self(value as u32)
    }
}

impl const From<PageTableEntry> for usize {
    fn from(value: PageTableEntry) -> Self {
        value.0 as Self
    }
}

impl PageTableEntry {
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }
}
