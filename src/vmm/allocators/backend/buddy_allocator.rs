use core::ptr::NonNull;

use crate::{
    bitmap::BitMap,
    vmm::{allocators::kmalloc::KfreeError, paging::PAGE_SIZE},
};

#[cfg(all(not(test), not(feature = "test-utils")))]
pub const BUDDY_ALLOCATOR_SIZE: usize = 1 << 29;

#[cfg(any(test, feature = "test-utils"))]
pub const BUDDY_ALLOCATOR_SIZE: usize = 1 << 25;

pub enum BuddyAllocationError {
    NotEnoughMemory,
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BuddyAllocatorNode {
    Free = 0b00,
    PartiallyAllocated = 0b10,
    FullyAllocated = 0b11,
}

impl core::fmt::Display for BuddyAllocatorNode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("0b{:02b}", u8::from(self)))
    }
}

impl const From<u8> for BuddyAllocatorNode {
    fn from(value: u8) -> Self {
        match value {
            0b00 => Self::Free,
            0b10 => Self::PartiallyAllocated,
            0b11 => Self::FullyAllocated,
            _ => unreachable!(),
        }
    }
}

impl const From<&BuddyAllocatorNode> for u8 {
    fn from(value: &BuddyAllocatorNode) -> Self {
        match value {
            BuddyAllocatorNode::Free => 0b00,
            BuddyAllocatorNode::PartiallyAllocated => 0b10,
            BuddyAllocatorNode::FullyAllocated => 0b11,
        }
    }
}

impl const From<BuddyAllocatorNode> for u8 {
    fn from(value: BuddyAllocatorNode) -> Self {
        value as u8
    }
}

macro_rules! bitmap_ptr_cast_mut {
    ($self:expr, $level:expr, |$bitmap:ident| $body:expr, $size:expr) => {{
        let $bitmap = unsafe { &mut *$self.levels[$level].cast::<BitMap<$size, 4>>().as_ptr() };
        $body
    }};
}

macro_rules! generate_bitmap_match_arms {
    ($self:expr, $level:expr, |$bitmap:ident| $body:expr, [$($lv:literal),* $(,)?]) => {
        match $level {
            0..=2 => bitmap_ptr_cast_mut!($self, $level, |$bitmap| $body, 8),
            $(
                $lv => bitmap_ptr_cast_mut!($self, $level, |$bitmap| $body, { 1 << $lv }),
            )*
            _ => unreachable!("BuddyAllocatorBitmap only has 20 levels (indices 0..=19)"),
        }
    };
}

/// Gets the bitmap from `self` (`BuddyAllocatorBitmap`) for `level`, casting it
/// to the correct type based on its size.
macro_rules! with_bitmap_at_level {
    ($self:expr, $level:expr, |$bitmap:ident| $body:expr) => {
        generate_bitmap_match_arms!($self, $level, |$bitmap| $body, [3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19])
    };
}

pub const MAX_BUDDY_ALLOCATOR_LEVELS: usize = ((1u64 << 32).ilog2() - 4096u64.ilog2()) as usize;

pub struct BuddyAllocator {
    /// Stores all possible levels of the bitmap. In order to span 4GiB with page
    /// granularity (where the root block is 4GiB, and the leaf nodes are 4096B),
    /// we need 20 levels (`log2(4294967296) - log2(4096)`).
    levels: [NonNull<u8>; MAX_BUDDY_ALLOCATOR_LEVELS],
    /// Address from which the memory block managed by the `BuddyAllocator` starts.
    root: Option<NonNull<u8>>,
    /// Index of the root bitmap (if `size == 4GiB`, use the full span of the tree
    /// (`root_level = 0`),  if `size == 2GiB`, start one level below (`root_level = 1`)
    /// , and so on).
    root_level: usize,
    /// Size of the root block.
    size: usize,
}

impl BuddyAllocator {
    #[allow(static_mut_refs)]
    pub const fn new(root: Option<NonNull<u8>>, size: usize, levels: [NonNull<u8>; MAX_BUDDY_ALLOCATOR_LEVELS]) -> Self {
        assert!(2usize.pow(size.ilog2()) == size, "size must be a power of 2");
        assert!(size >= 1 << 15 && size <= usize::MAX, "size must be at least 32768 and at most 2147483648");

        let root_level = 31 - size.ilog2() as usize;

        Self {
            levels,
            root,
            root_level,
            size,
        }
    }

    pub fn set_root(&mut self, root: NonNull<u8>) {
        self.root = Some(root);
    }

    #[inline]
    #[allow(static_mut_refs)]
    fn alloc_internal(&mut self, allocation_size: usize, root: *const u8, level_block_size: usize, level: usize, index: usize) -> Option<*mut u8> {
        assert!(
            allocation_size.is_multiple_of(PAGE_SIZE),
            "The buddy allocator can only allocate multiples of 0x1000"
        );

        let current_state = with_bitmap_at_level!(self, level, |bitmap| bitmap.get(index));
        if current_state == BuddyAllocatorNode::FullyAllocated as u8 {
            return None;
        }

        if allocation_size >= level_block_size || level == self.levels.len() {
            if current_state == BuddyAllocatorNode::Free as u8 {
                with_bitmap_at_level!(self, level, |bitmap| bitmap.set(index, BuddyAllocatorNode::FullyAllocated as u8));
                return Some(root as *mut u8);
            }
            return None;
        }

        let left_child_index = index * 2;
        let allocation = self.alloc_internal(allocation_size, root, level_block_size / 2, level + 1, left_child_index);

        if allocation.is_some() {
            with_bitmap_at_level!(self, level, |bitmap| {
                let state = bitmap.get(index);
                if state == BuddyAllocatorNode::Free as u8 {
                    bitmap.set(index, BuddyAllocatorNode::PartiallyAllocated as u8);
                }
            });
            return allocation;
        }

        let right_child_index = index * 2 + 1;
        let allocation = self.alloc_internal(
            allocation_size,
            (root as usize + level_block_size / 2) as *const u8,
            level_block_size / 2,
            level + 1,
            right_child_index,
        );

        if allocation.is_some() {
            with_bitmap_at_level!(self, level, |bitmap| {
                let state = bitmap.get(index);
                if state == BuddyAllocatorNode::Free as u8 {
                    bitmap.set(index, BuddyAllocatorNode::PartiallyAllocated as u8);
                }
            });
        }

        allocation
    }

    pub fn alloc(&mut self, size: usize) -> Result<*mut u8, BuddyAllocationError> {
        assert!(size.is_multiple_of(PAGE_SIZE), "The buddy allocator can only allocate multiples of 0x1000");
        assert!(size <= self.size, "The buddy allocator cannot allocate more than its size");
        let root = self.root.expect("alloc called on BuddyAllocator without root");

        self.alloc_internal(size, root.as_ptr(), self.size, self.root_level, 0)
            .ok_or(BuddyAllocationError::NotEnoughMemory)
    }

    /// Gets the base index (level 19, page granularity) for a given `addr`.
    /// Used to recurse back from there and find an allocation by address.
    #[inline]
    fn get_base_index(&self, addr: *const u8) -> usize {
        let root = self.root.expect("get_base_index called on BuddyAllocator without root");
        assert!(
            (root.as_ptr() as usize..(root.as_ptr() as usize + self.size)).contains(&(addr as usize)),
            "addr is out of range for this allocator"
        );

        (addr as usize - root.as_ptr() as usize) / PAGE_SIZE
    }

    fn update_parent_states(&mut self, level: usize, index: usize) {
        if level == self.root_level {
            return;
        }

        let parent_index = index / 2;

        let left_state = with_bitmap_at_level!(self, level, |bitmap| bitmap.get(index & !1));
        let right_state = with_bitmap_at_level!(self, level, |bitmap| bitmap.get(index | 1));

        let parent_state = match (left_state, right_state) {
            (0b00, 0b00) => BuddyAllocatorNode::Free,
            (0b11, 0b11) => BuddyAllocatorNode::FullyAllocated,
            _ => BuddyAllocatorNode::PartiallyAllocated,
        };

        with_bitmap_at_level!(self, level - 1, |bitmap| bitmap.set(parent_index, parent_state as u8));

        self.update_parent_states(level - 1, parent_index);
    }

    fn coalesce(&mut self, level: usize, index: usize) {
        if level == self.root_level {
            return;
        }

        let buddy_index = if index % 2 == 1 { index - 1 } else { index + 1 };
        let buddy_state = with_bitmap_at_level!(self, level, |bitmap| bitmap.get(buddy_index));

        let parent_index = index / 2;
        if buddy_state == BuddyAllocatorNode::Free as u8 {
            with_bitmap_at_level!(self, level - 1, |bitmap| bitmap.set(parent_index, BuddyAllocatorNode::Free as u8));
            self.coalesce(level - 1, index);
        } else {
            with_bitmap_at_level!(self, level - 1, |bitmap| {
                let parent_state = bitmap.get(parent_index);
                if parent_state == BuddyAllocatorNode::FullyAllocated as u8 {
                    bitmap.set(parent_index, BuddyAllocatorNode::PartiallyAllocated as u8);
                }
            });

            if level > self.root_level + 1 {
                self.update_parent_states(level - 1, parent_index);
            }
        }
    }

    pub fn free(&mut self, addr: *const u8) -> Result<(), KfreeError> {
        assert!(!addr.is_null(), "Cannot free null pointer");
        assert!(self.root.is_some(), "free called on BuddyAllocator without root");

        let mut index = self.get_base_index(addr);

        for level in (self.root_level..=self.levels.len() - 1).rev() {
            let state = with_bitmap_at_level!(self, level, |bitmap| bitmap.get(index));
            if state == 0b11 {
                with_bitmap_at_level!(self, level, |bitmap| bitmap.set(index, BuddyAllocatorNode::Free as u8));
                self.coalesce(level, index);
                return Ok(());
            }
            index = if index % 2 == 1 { index - 1 } else { index } / 2;
        }
        Err(KfreeError::InvalidPointer)
    }
}
