use crate::{bitmap::BitMap, vmm::allocators::backend::buddy_allocator::MAX_BUDDY_ALLOCATOR_LEVELS};

pub static mut LEVEL_0: BitMap<8, 4> = BitMap::<8, 4>::new();
pub static mut LEVEL_1: BitMap<8, 4> = BitMap::<8, 4>::new();
pub static mut LEVEL_2: BitMap<8, 4> = BitMap::<8, 4>::new();
pub static mut LEVEL_3: BitMap<{ 1 << 3 }, 4> = BitMap::<{ 1 << 3 }, 4>::new();
pub static mut LEVEL_4: BitMap<{ 1 << 4 }, 4> = BitMap::<{ 1 << 4 }, 4>::new();
pub static mut LEVEL_5: BitMap<{ 1 << 5 }, 4> = BitMap::<{ 1 << 5 }, 4>::new();
pub static mut LEVEL_6: BitMap<{ 1 << 6 }, 4> = BitMap::<{ 1 << 6 }, 4>::new();
pub static mut LEVEL_7: BitMap<{ 1 << 7 }, 4> = BitMap::<{ 1 << 7 }, 4>::new();
pub static mut LEVEL_8: BitMap<{ 1 << 8 }, 4> = BitMap::<{ 1 << 8 }, 4>::new();
pub static mut LEVEL_9: BitMap<{ 1 << 9 }, 4> = BitMap::<{ 1 << 9 }, 4>::new();
pub static mut LEVEL_10: BitMap<{ 1 << 10 }, 4> = BitMap::<{ 1 << 10 }, 4>::new();
pub static mut LEVEL_11: BitMap<{ 1 << 11 }, 4> = BitMap::<{ 1 << 11 }, 4>::new();
pub static mut LEVEL_12: BitMap<{ 1 << 12 }, 4> = BitMap::<{ 1 << 12 }, 4>::new();
pub static mut LEVEL_13: BitMap<{ 1 << 13 }, 4> = BitMap::<{ 1 << 13 }, 4>::new();
pub static mut LEVEL_14: BitMap<{ 1 << 14 }, 4> = BitMap::<{ 1 << 14 }, 4>::new();
pub static mut LEVEL_15: BitMap<{ 1 << 15 }, 4> = BitMap::<{ 1 << 15 }, 4>::new();
pub static mut LEVEL_16: BitMap<{ 1 << 16 }, 4> = BitMap::<{ 1 << 16 }, 4>::new();
pub static mut LEVEL_17: BitMap<{ 1 << 17 }, 4> = BitMap::<{ 1 << 17 }, 4>::new();
pub static mut LEVEL_18: BitMap<{ 1 << 18 }, 4> = BitMap::<{ 1 << 18 }, 4>::new();
pub static mut LEVEL_19: BitMap<{ 1 << 19 }, 4> = BitMap::<{ 1 << 19 }, 4>::new();
pub static mut LEVEL_20: BitMap<{ 1 << 20 }, 4> = BitMap::<{ 1 << 20 }, 4>::new();

#[allow(static_mut_refs)]
pub static mut LEVELS: [*const u8; MAX_BUDDY_ALLOCATOR_LEVELS] = unsafe {
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
        // LEVEL_20.as_ptr(),
    ]
};
