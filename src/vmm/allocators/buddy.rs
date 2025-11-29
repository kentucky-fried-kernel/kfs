use crate::vmm::{allocators::bitmap::BitMap, paging};

// TODO: Each enum variant takes as much space as the largest one, this is a stupid solution.
// Trait objects are not an option since they do not support const initialization, will probably have
// to resort to pointers here.
// enum BitMapLevel {
//     Level0 { map: BitMap<8> },
//     Level1 { map: BitMap<8> },
//     Level2 { map: BitMap<8> },
//     Level3 { map: BitMap<{ 1 << 3 }> },
//     Level4 { map: BitMap<{ 1 << 4 }> },
//     Level5 { map: BitMap<{ 1 << 5 }> },
//     Level6 { map: BitMap<{ 1 << 6 }> },
//     Level7 { map: BitMap<{ 1 << 7 }> },
//     Level8 { map: BitMap<{ 1 << 8 }> },
//     Level9 { map: BitMap<{ 1 << 9 }> },
//     Level10 { map: BitMap<{ 1 << 10 }> },
//     Level11 { map: BitMap<{ 1 << 11 }> },
//     Level12 { map: BitMap<{ 1 << 12 }> },
//     Level13 { map: BitMap<{ 1 << 13 }> },
//     Level14 { map: BitMap<{ 1 << 14 }> },
//     Level15 { map: BitMap<{ 1 << 15 }> },
//     Level16 { map: BitMap<{ 1 << 16 }> },
//     Level17 { map: BitMap<{ 1 << 17 }> },
//     Level18 { map: BitMap<{ 1 << 18 }> },
//     Level19 { map: BitMap<{ 1 << 19 }> },
// }

static mut LEVEL_0: BitMap<8> = BitMap::<8>::new();
static mut LEVEL_1: BitMap<8> = BitMap::<8>::new();
static mut LEVEL_2: BitMap<8> = BitMap::<8>::new();
static mut LEVEL_3: BitMap<{ 1 << 3 }> = BitMap::<{ 1 << 3 }>::new();
static mut LEVEL_4: BitMap<{ 1 << 4 }> = BitMap::<{ 1 << 4 }>::new();
static mut LEVEL_5: BitMap<{ 1 << 5 }> = BitMap::<{ 1 << 5 }>::new();
static mut LEVEL_6: BitMap<{ 1 << 6 }> = BitMap::<{ 1 << 6 }>::new();
static mut LEVEL_7: BitMap<{ 1 << 7 }> = BitMap::<{ 1 << 7 }>::new();
static mut LEVEL_8: BitMap<{ 1 << 8 }> = BitMap::<{ 1 << 8 }>::new();
static mut LEVEL_9: BitMap<{ 1 << 9 }> = BitMap::<{ 1 << 9 }>::new();
static mut LEVEL_10: BitMap<{ 1 << 10 }> = BitMap::<{ 1 << 10 }>::new();
static mut LEVEL_11: BitMap<{ 1 << 11 }> = BitMap::<{ 1 << 11 }>::new();
static mut LEVEL_12: BitMap<{ 1 << 12 }> = BitMap::<{ 1 << 12 }>::new();
static mut LEVEL_13: BitMap<{ 1 << 13 }> = BitMap::<{ 1 << 13 }>::new();
static mut LEVEL_14: BitMap<{ 1 << 14 }> = BitMap::<{ 1 << 14 }>::new();
static mut LEVEL_15: BitMap<{ 1 << 15 }> = BitMap::<{ 1 << 15 }>::new();
static mut LEVEL_16: BitMap<{ 1 << 16 }> = BitMap::<{ 1 << 16 }>::new();
static mut LEVEL_17: BitMap<{ 1 << 17 }> = BitMap::<{ 1 << 17 }>::new();
static mut LEVEL_18: BitMap<{ 1 << 18 }> = BitMap::<{ 1 << 18 }>::new();
static mut LEVEL_19: BitMap<{ 1 << 19 }> = BitMap::<{ 1 << 19 }>::new();

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
        let log2 = size.ilog2();

        assert!(2usize.pow(log2) == size, "size parameter must be a power of 2");
        assert!(size >= 1 << 15 && size <= i32::MAX as usize, "size parameter must be at least 32768 and at most 2147483648");

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

        let root_level = match size {
            _ if size == 1 << 31 => 1,
            _ if size == 1 << 30 => 2,
            _ if size == 1 << 29 => 3,
            _ if size == 1 << 28 => 4,
            _ if size == 1 << 27 => 5,
            _ if size == 1 << 26 => 6,
            _ if size == 1 << 25 => 7,
            _ if size == 1 << 24 => 8,
            _ if size == 1 << 23 => 9,
            _ if size == 1 << 22 => 10,
            _ if size == 1 << 21 => 11,
            _ if size == 1 << 20 => 12,
            _ if size == 1 << 19 => 13,
            _ if size == 1 << 18 => 14,
            _ if size == 1 << 17 => 15,
            _ if size == 1 << 16 => 16,
            _ if size == 1 << 15 => 17,
            _ => unreachable!(),
        };

        Self { levels, root, root_level, size }
    }

    fn alloc_internal(&mut self, allocation_size: usize, root: *const u8, level_block_size: usize) -> *const u8 {
        // figure out if we go left or right
        if level_block_size / 2 >= allocation_size {
            return self.alloc_internal(allocation_size, root, level_block_size / 2);
        }
        0 as *const u8
    }

    pub fn alloc(&mut self, size: usize) -> *const u8 {
        assert_eq!(size % paging::PAGE_SIZE, 0, "The buddy allocator can only allocate multiples of PAGE_SIZE ({})", paging::PAGE_SIZE);

        if size > self.size {
            // probably dynamically grow here?
        }

        self.alloc_internal(size, self.root, self.size)
    }
}
