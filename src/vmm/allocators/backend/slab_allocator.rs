use crate::{bitmap::BitMap, vmm::paging::PAGE_SIZE};

/// BitMap type specifically for slabs, which may only store up to 512 entries
/// (for `object_size == 8`). This wastes a little bit of memory for larger
/// `object_size` values, but I think the simplicity is worth it here.
type SlabBitMap = BitMap<{ PAGE_SIZE / 8 }, 8>;

#[repr(u8)]
pub enum SlabObjectInfo {
    Free = 0,
    Allocated = 1,
}

pub enum SlabAllocationError {
    NotEnoughMemory,
}

pub enum SlabFreeError {
    InvalidPointer,
}

impl From<u8> for SlabObjectInfo {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Free,
            1 => Self::Allocated,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Slab {
    addr: *const u8,
    object_size: usize,
    bitmap: SlabBitMap,
    /// Tracks the number of allocated objects in this slab. Allows prioritizing
    /// allocations from fuller slabs to maximize the number of empty slabs to give
    /// back to the main allocator in case of memory pressure.
    allocated: usize,
}

impl Slab {
    /// Creates a `Slab` object from `addr` and `object_size`.
    /// # Safety
    /// It is the caller's responsibility to ensure that `addr` points to a valid,
    /// page-aligned address, with at least 4096 read-writable bytes.
    pub unsafe fn new(addr: *const u8, object_size: usize) -> Self {
        assert!(addr.is_aligned_to(PAGE_SIZE), "addr is not page-aligned");

        Self {
            addr,
            object_size,
            bitmap: SlabBitMap::new(),
            allocated: 0,
        }
    }

    #[inline]
    fn max_objects(&self) -> usize {
        PAGE_SIZE / self.object_size
    }

    /// Returns a pointer to a free memory region of size `object_size`, or a
    /// `SlabAllocationError` if no more space is left.
    pub fn alloc(&mut self) -> Result<*const u8, SlabAllocationError> {
        if self.allocated == self.max_objects() {
            return Err(SlabAllocationError::NotEnoughMemory);
        }

        let (idx, (ptr, _)) = self
            .into_iter()
            .enumerate()
            .find(|(_, (_, status))| match status {
                SlabObjectInfo::Allocated => false,
                SlabObjectInfo::Free => true,
            })
            .ok_or(SlabAllocationError::NotEnoughMemory)?;

        self.bitmap.set(idx, 1);

        Ok(ptr)
    }

    pub fn free(&mut self, addr: *const u8) -> Result<(), SlabFreeError> {
        assert!(
            addr as usize >= self.addr as usize && (addr as usize) < self.addr as usize + PAGE_SIZE,
            "addr is out of range for this slab"
        );

        for (idx, (ptr, status)) in self.into_iter().enumerate() {
            if ptr == addr {
                match status {
                    SlabObjectInfo::Allocated => {
                        self.bitmap.set(idx, SlabObjectInfo::Free as u8);
                        return Ok(());
                    }
                    SlabObjectInfo::Free => return Err(SlabFreeError::InvalidPointer),
                }
            }
        }
        Err(SlabFreeError::InvalidPointer)
    }
}

impl IntoIterator for Slab {
    type Item = (*const u8, SlabObjectInfo);
    type IntoIter = SlabIntoIterator;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter { slab: self, pos: 0 }
    }
}

pub struct SlabIntoIterator {
    slab: Slab,
    pos: usize,
}

impl Iterator for SlabIntoIterator {
    type Item = (*const u8, SlabObjectInfo);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos + self.slab.object_size >= (self.pos & !0xFFF) + 0x1000 {
            return None;
        }

        let current_addr = unsafe { self.slab.addr.add(self.slab.object_size * self.pos) };
        let status = SlabObjectInfo::from(self.slab.bitmap.get(self.pos));

        self.pos += 1;

        Some((current_addr, status))
    }
}
