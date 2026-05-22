use alloc::vec::Vec;

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Permissions {
    Read = 0,
    ReadWrite = 1,
}

pub struct Segment {
    pub offset: Option<usize>,
    pub vaddr: usize,
    pub size: usize,
    pub permissions: Permissions,
}

pub struct Binary {
    pub segments: Vec<Segment>,
    pub entry: usize,
    pub stack: usize,
}

impl Binary {
    #[must_use]
    pub fn new(entry: usize, stack: usize) -> Self {
        Self {
            segments: Vec::new(),
            entry,
            stack,
        }
    }
}
