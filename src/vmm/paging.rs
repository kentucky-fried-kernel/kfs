pub mod conversion;
pub mod init;
pub mod mmap;
pub mod page_entries;
pub mod state;

pub const PAGE_SIZE: usize = 0x1000;

#[derive(Clone, Copy, PartialEq)]
pub enum Permissions {
    Read = 0,
    ReadWrite = 1,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Access {
    User,
    Root,
}
