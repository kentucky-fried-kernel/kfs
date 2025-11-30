use crate::{
    printkln,
    vmm::{allocators::bitmap::BitMap, paging::PAGE_SIZE},
};

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

macro_rules! bitmap_ptr_cast_mut {
    ($self:expr, $level:expr, |$bitmap:ident| $body:expr, $size:expr) => {{
        let $bitmap = unsafe { &mut *$self.levels[$level].cast::<BitMap<$size, 4>>().cast_mut() };
        $body
    }};
}

macro_rules! generate_bitmap_match_arms {
    ($self:expr, $level:expr, |$bitmap:ident| $body:expr, [$($lv:literal),* $(,)?]) => {
        match $level {
            0 | 1 | 2 => bitmap_ptr_cast_mut!($self, $level, |$bitmap| $body, 8),
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

pub struct BuddyAllocatorBitmap {
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

impl BuddyAllocatorBitmap {
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

        let root_level = Self::get_root_level(size);

        Self {
            levels,
            root,
            root_level,
            size,
        }
    }

    const fn get_root_level(size: usize) -> usize {
        assert!(size >= 1 << 15 && size <= 1 << 31, "size must be at least 32768 and at most 2147483648");

        31 - size.ilog2() as usize
    }

    #[inline]
    #[allow(static_mut_refs)]
    fn alloc_internal(&mut self, allocation_size: usize, root: *const u8, level_block_size: usize, level: usize, index: usize) -> Option<*const u8> {
        assert!(allocation_size % PAGE_SIZE == 0, "The buddy allocator can only allocate multiples of 4096");

        // Special handling for bitmap.len() == 1
        if level == self.root_level {
            // This means we need to allocate the entire block if it is free
            if allocation_size > level_block_size / 2 {
                if with_bitmap_at_level!(self, level, |bitmap| bitmap.get(index)) == 1 {
                    // The entire memory needs to be free for this allocation to be possible
                    return None;
                }
                return Some(root);
            }
            return self.alloc_internal(allocation_size, root, level_block_size / 2, level + 1, index);
        }

        let (left, right) = with_bitmap_at_level!(self, level, |bitmap| {
            printkln!("{}", bitmap.get(index));
            bitmap.set(index, 0b10);
            printkln!("{}", bitmap.get(index));
            bitmap.clear(index);
            printkln!("{}", bitmap.get(index));

            (bitmap.get(index), bitmap.get(index + 1))
        });

        if allocation_size > level_block_size / 2 {
            match (left, right) {
                (0, _) => return Some(root),
                (_, 0) => return Some((root as usize + level_block_size) as *const u8),
                _ => return None,
            }
        }

        if level == self.levels.len() {
            return None;
        }
        let next_index = match level {
            0 => 0,
            _ => index * 2 + if index % 2 == 0 { 0 } else { 1 },
        };

        if level_block_size / 2 >= allocation_size {
            return self.alloc_internal(allocation_size, root, level_block_size / 2, level + 1, next_index);
        }

        None
    }

    pub fn alloc(&mut self, size: usize) -> Option<*const u8> {
        assert!(size % PAGE_SIZE == 0, "The buddy allocator can only allocate multiples of 4096");
        assert!(size < self.size, "The buddy allocator cannot allocate more than {}", self.size);

        // if size > self.size {
        //     // probably dynamically grow here?
        //     // - would have to grow by a factor of at least 4 to satisfy the request,
        //     //   might make more sense to just pass on the request to mmap directly
        //     //   return mmap(size, ...);
        // }

        self.alloc_internal(size, self.root, self.size, self.root_level, 0)
        // 0 as *const u8
    }
}
