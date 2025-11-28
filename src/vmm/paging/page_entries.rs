pub const PAGE_DIRECTORY_SIZE: usize = 1024;
#[repr(align(0x1000))]
pub struct PageDirectory(pub [PageDirectoryEntry; PAGE_DIRECTORY_SIZE]);

// pub struct  OutOfBoundsError;

// impl PageDirectory {
//     pub const fn new(entries: &'static mut [PageDirectoryEntry; PAGE_DIRECTORY_SIZE]) -> Self {
//         Self { entries }
//     }

//     pub fn set_entry(&mut self, index: usize, entry: PageDirectoryEntry) -> Result<(), OutOfBoundsError> {
//         if index >= self.entries.len() {
//             return Err(OutOfBoundsError);
//         }

//         self.entries[index] = entry;

//         Ok(())
//     }
//     pub fn get_entry(&'static self, index: usize) -> Option<&'static mut PageDirectoryEntry> {
//         if index >= self.entries.len() {
//             return None;
//         }

//         Some(&self.entries[index])
//     }
//     pub fn get_entry_mut(&mut self, index: usize) -> Option<&mut PageDirectoryEntry> {
//         if index >= self.entries.len() {
//             return None;
//         }

//         Some(&mut self.entries[index])
//     }
// }

// pub struct PageDirectoryIntoIterator {
//     page_directory: &'static mut PageDirectory,
//     pos: usize,
// }

// impl Iterator for PageDirectoryIntoIterator {
//     type Item = &'static mut PageDirectoryEntry;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.pos >= self.page_directory.entries.len() {
//             return None;
//         }

//         let cur_pos = self.pos;
//         self.pos += 1;
//         let p: &'static mut PageDirectoryEntry = self.page_directory.get_entry(cur_pos).unwrap();
//         Some(p)
//     }
// }

// impl IntoIterator for &'static mut PageDirectory {
//     type Item = &'static mut PageDirectoryEntry;
//     type IntoIter = PageDirectoryIntoIterator;

//     fn into_iter(self) -> Self::IntoIter {
//         Self::IntoIter { page_directory: self, pos: 0}
//     }
// }

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
    pub const fn empty() -> Self {
        unsafe { core::mem::transmute::<usize, PageDirectoryEntry>(0) }
    }

    pub const fn from_usize(value: usize) -> Self {
        unsafe { core::mem::transmute::<usize, PageDirectoryEntry>(value) }
    }

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
    pub const fn empty() -> Self {
        unsafe { core::mem::transmute::<usize, Self>(0) }
    }

    pub const fn from_usize(value: usize) -> Self {
        unsafe { core::mem::transmute::<usize, Self>(value) }
    }

    pub const fn to_usize(&self) -> usize {
        unsafe { core::mem::transmute::<Self, usize>(*self) }
    }
}
