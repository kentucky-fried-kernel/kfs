use crate::{bitmap::BitMap, printkln, vmm::paging::PAGE_SIZE};

static mut LEVEL_0: BitMap<8, 4> = BitMap::<8, 4>::new();
static mut LEVEL_1: BitMap<8, 4> = BitMap::<8, 4>::new();
static mut LEVEL_2: BitMap<8, 4> = BitMap::<8, 4>::new();
static mut LEVEL_3: BitMap<{ 1 << 3 }, 4> = BitMap::<{ 1 << 3 }, 4>::new();
static mut LEVEL_4: BitMap<{ 1 << 4 }, 4> = BitMap::<{ 1 << 4 }, 4>::new();
static mut LEVEL_5: BitMap<{ 1 << 5 }, 4> = BitMap::<{ 1 << 5 }, 4>::new();
static mut LEVEL_6: BitMap<{ 1 << 6 }, 4> = BitMap::<{ 1 << 6 }, 4>::new();
static mut LEVEL_7: BitMap<{ 1 << 7 }, 4> = BitMap::<{ 1 << 7 }, 4>::new();
static mut LEVEL_8: BitMap<{ 1 << 8 }, 4> = BitMap::<{ 1 << 8 }, 4>::new();
static mut LEVEL_9: BitMap<{ 1 << 9 }, 4> = BitMap::<{ 1 << 9 }, 4>::new();
static mut LEVEL_10: BitMap<{ 1 << 10 }, 4> = BitMap::<{ 1 << 10 }, 4>::new();
static mut LEVEL_11: BitMap<{ 1 << 11 }, 4> = BitMap::<{ 1 << 11 }, 4>::new();
static mut LEVEL_12: BitMap<{ 1 << 12 }, 4> = BitMap::<{ 1 << 12 }, 4>::new();
static mut LEVEL_13: BitMap<{ 1 << 13 }, 4> = BitMap::<{ 1 << 13 }, 4>::new();
static mut LEVEL_14: BitMap<{ 1 << 14 }, 4> = BitMap::<{ 1 << 14 }, 4>::new();
static mut LEVEL_15: BitMap<{ 1 << 15 }, 4> = BitMap::<{ 1 << 15 }, 4>::new();
static mut LEVEL_16: BitMap<{ 1 << 16 }, 4> = BitMap::<{ 1 << 16 }, 4>::new();
static mut LEVEL_17: BitMap<{ 1 << 17 }, 4> = BitMap::<{ 1 << 17 }, 4>::new();
static mut LEVEL_18: BitMap<{ 1 << 18 }, 4> = BitMap::<{ 1 << 18 }, 4>::new();
static mut LEVEL_19: BitMap<{ 1 << 19 }, 4> = BitMap::<{ 1 << 19 }, 4>::new();

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
        let $bitmap = unsafe { &mut *$self.levels[$level].cast::<BitMap<$size, 4>>().cast_mut() };
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
            _ => unreachable!("BuddyAllocatorBitmap only has 20 levels (indices 0..19)"),
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

pub struct BuddyAllocator {
    /// `levels[0]`: 1 * 2 GiB
    ///
    /// `levels[19]`: 1.048.576 * 4096 B
    levels: [*const u8; 20],
    /// Address from which the root block starts.
    root: *const u8,
    /// Index of the root bitmap
    root_level: usize,
    /// Size of the root block.
    size: usize,
}

impl BuddyAllocator {
    #[allow(static_mut_refs)]
    pub const fn new(root: *const u8, size: usize) -> Self {
        assert!(2usize.pow(size.ilog2()) == size, "size must be a power of 2");
        assert!(size >= 1 << 15 && size <= 1 << 31, "size must be at least 32768 and at most 2147483648");

        let levels = unsafe {
            [
                LEVEL_0.as_ptr(),
                LEVEL_1.as_ptr(),
                LEVEL_2.as_ptr(),
                LEVEL_3.as_ptr(),
                LEVEL_4.as_ptr(),
                LEVEL_5.as_ptr(),
                LEVEL_6.as_ptr(),
                LEVEL_7.as_ptr(),
                LEVEL_8.as_ptr(),
                LEVEL_9.as_ptr(),
                LEVEL_10.as_ptr(),
                LEVEL_11.as_ptr(),
                LEVEL_12.as_ptr(),
                LEVEL_13.as_ptr(),
                LEVEL_14.as_ptr(),
                LEVEL_15.as_ptr(),
                LEVEL_16.as_ptr(),
                LEVEL_17.as_ptr(),
                LEVEL_18.as_ptr(),
                LEVEL_19.as_ptr(),
            ]
        };

        let root_level = 31 - size.ilog2() as usize;

        Self {
            levels,
            root,
            root_level,
            size,
        }
    }

    #[inline]
    #[allow(static_mut_refs)]
    fn alloc_internal(&mut self, allocation_size: usize, root: *const u8, level_block_size: usize, level: usize, index: usize) -> Option<*const u8> {
        assert!(
            allocation_size.is_multiple_of(PAGE_SIZE),
            "The buddy allocator can only allocate multiples of 4096"
        );

        let current_state = with_bitmap_at_level!(self, level, |bitmap| bitmap.get(index));
        if current_state == BuddyAllocatorNode::FullyAllocated as u8 {
            return None;
        }

        if allocation_size >= level_block_size || level == self.levels.len() {
            if current_state == BuddyAllocatorNode::Free as u8 {
                with_bitmap_at_level!(self, level, |bitmap| bitmap.set(index, 0b11));
                return Some(root);
            }
            return None;
        }

        let left_child_index = index * 2;
        let allocation = self.alloc_internal(allocation_size, root, level_block_size / 2, level + 1, left_child_index);

        if allocation.is_some() {
            with_bitmap_at_level!(self, level, |bitmap| {
                let state = bitmap.get(index);
                if state == 0b00 {
                    bitmap.set(index, 0b10);
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
                if state == 0b00 {
                    bitmap.set(index, 0b10);
                }
            });
        }

        allocation
    }

    pub fn alloc(&mut self, size: usize) -> Option<*const u8> {
        assert!(size.is_multiple_of(PAGE_SIZE), "The buddy allocator can only allocate multiples of 4096");
        assert!(size < self.size, "The buddy allocator cannot allocate more than its size");

        self.alloc_internal(size, self.root, self.size, self.root_level, 0)
    }

    /// Gets the base index (level 19, page granularity) for a given `addr`.
    /// Used to recurse back from there and find an allocation by address.
    #[inline]
    fn get_base_index(&self, addr: *const u8) -> usize {
        assert!(
            (self.root as usize..(self.root as usize + self.size)).contains(&(addr as usize)),
            "addr is out of range for this allocator"
        );

        (addr as usize - self.root as usize) / PAGE_SIZE
    }

    fn update_parent_states(&mut self, level: usize, index: usize) {
        if level == self.root_level {
            return;
        }

        let parent_index = index / 2;

        let left_state = with_bitmap_at_level!(self, level, |bitmap| bitmap.get(index & !1));
        let right_state = with_bitmap_at_level!(self, level, |bitmap| bitmap.get(index | 1));

        let parent_state = match (left_state, right_state) {
            (0b00, 0b00) => 0b00,
            (0b11, 0b11) => 0b11,
            _ => 0b10,
        };

        with_bitmap_at_level!(self, level - 1, |bitmap| bitmap.set(parent_index, parent_state));

        self.update_parent_states(level - 1, parent_index);
    }

    fn coalesce(&mut self, level: usize, index: usize) {
        if level == self.root_level {
            return;
        }

        let buddy_index = if index % 2 == 1 { index - 1 } else { index + 1 };
        let buddy_state = with_bitmap_at_level!(self, level, |bitmap| bitmap.get(buddy_index));

        let parent_index = index / 2;
        if buddy_state == 0b00 {
            with_bitmap_at_level!(self, level - 1, |bitmap| bitmap.set(parent_index, 0b00));
            self.coalesce(level - 1, index);
        } else {
            with_bitmap_at_level!(self, level - 1, |bitmap| {
                let parent_state = bitmap.get(parent_index);
                if parent_state == 0b11 {
                    bitmap.set(parent_index, 0b10);
                }
            });

            if level > self.root_level + 1 {
                self.update_parent_states(level - 1, parent_index);
            }
        }
    }

    pub fn free(&mut self, addr: *const u8) {
        assert!(!addr.is_null(), "Cannot free null pointer");

        printkln!("Freeing 0x{:x}", addr as usize);
        let mut index = self.get_base_index(addr);

        for level in (self.root_level..=self.levels.len() - 1).rev() {
            let state = with_bitmap_at_level!(self, level, |bitmap| bitmap.get(index));
            if state == 0b11 {
                with_bitmap_at_level!(self, level, |bitmap| bitmap.set(index, 0b00));
                self.coalesce(level, index);
                return;
            }
            index = if index % 2 == 1 { index - 1 } else { index } / 2;
        }
    }
}
