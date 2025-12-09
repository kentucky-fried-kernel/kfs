pub const PAGE_DIRECTORY_SIZE: usize = 1024;
#[repr(align(0x1000))]
pub struct PageDirectory(pub [PageDirectoryEntry; PAGE_DIRECTORY_SIZE]);

pub const PAGE_TABLE_SIZE: usize = 1024;
#[repr(align(0x1000))]
#[derive(Clone, Copy)]
pub struct PageTable(pub [PageTableEntry; PAGE_TABLE_SIZE]);

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

impl PageDirectoryEntry {
    #[must_use]
    pub const fn empty() -> Self {
        unsafe { core::mem::transmute::<usize, PageDirectoryEntry>(0) }
    }

    #[must_use]
    pub const fn from_usize(value: usize) -> Self {
        unsafe { core::mem::transmute::<usize, PageDirectoryEntry>(value) }
    }

    #[must_use]
    pub const fn to_usize(&self) -> usize {
        unsafe { core::mem::transmute::<PageDirectoryEntry, usize>(*self) }
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

impl PageTableEntry {
    #[must_use]
    pub const fn empty() -> Self {
        unsafe { core::mem::transmute::<usize, Self>(0) }
    }

    #[must_use]
    pub const fn from_usize(value: usize) -> Self {
        unsafe { core::mem::transmute::<usize, Self>(value) }
    }

    #[must_use]
    pub const fn to_usize(&self) -> usize {
        unsafe { core::mem::transmute::<Self, usize>(*self) }
    }
}
