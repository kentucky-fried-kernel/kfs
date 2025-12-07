use crate::bitmap::Bitmap;

pub static mut LEVEL_0: Bitmap<8, 4> = Bitmap::<8, 4>::new();
pub static mut LEVEL_1: Bitmap<8, 4> = Bitmap::<8, 4>::new();
pub static mut LEVEL_2: Bitmap<8, 4> = Bitmap::<8, 4>::new();
pub static mut LEVEL_3: Bitmap<8, 4> = Bitmap::<8, 4>::new();
pub static mut LEVEL_4: Bitmap<{ 1 << 4 }, 4> = Bitmap::<{ 1 << 4 }, 4>::new();
pub static mut LEVEL_5: Bitmap<{ 1 << 5 }, 4> = Bitmap::<{ 1 << 5 }, 4>::new();
pub static mut LEVEL_6: Bitmap<{ 1 << 6 }, 4> = Bitmap::<{ 1 << 6 }, 4>::new();
pub static mut LEVEL_7: Bitmap<{ 1 << 7 }, 4> = Bitmap::<{ 1 << 7 }, 4>::new();
pub static mut LEVEL_8: Bitmap<{ 1 << 8 }, 4> = Bitmap::<{ 1 << 8 }, 4>::new();
pub static mut LEVEL_9: Bitmap<{ 1 << 9 }, 4> = Bitmap::<{ 1 << 9 }, 4>::new();
pub static mut LEVEL_10: Bitmap<{ 1 << 10 }, 4> = Bitmap::<{ 1 << 10 }, 4>::new();
pub static mut LEVEL_11: Bitmap<{ 1 << 11 }, 4> = Bitmap::<{ 1 << 11 }, 4>::new();
pub static mut LEVEL_12: Bitmap<{ 1 << 12 }, 4> = Bitmap::<{ 1 << 12 }, 4>::new();
pub static mut LEVEL_13: Bitmap<{ 1 << 13 }, 4> = Bitmap::<{ 1 << 13 }, 4>::new();
pub static mut LEVEL_14: Bitmap<{ 1 << 14 }, 4> = Bitmap::<{ 1 << 14 }, 4>::new();
pub static mut LEVEL_15: Bitmap<{ 1 << 15 }, 4> = Bitmap::<{ 1 << 15 }, 4>::new();
pub static mut LEVEL_16: Bitmap<{ 1 << 16 }, 4> = Bitmap::<{ 1 << 16 }, 4>::new();
pub static mut LEVEL_17: Bitmap<{ 1 << 17 }, 4> = Bitmap::<{ 1 << 17 }, 4>::new();
pub static mut LEVEL_18: Bitmap<{ 1 << 18 }, 4> = Bitmap::<{ 1 << 18 }, 4>::new();
pub static mut LEVEL_19: Bitmap<{ 1 << 19 }, 4> = Bitmap::<{ 1 << 19 }, 4>::new();
pub static mut LEVEL_20: Bitmap<{ 1 << 20 }, 4> = Bitmap::<{ 1 << 20 }, 4>::new();

/// Macro to construct the levels array inline, avoiding the need to store it in
/// a static (since we cannot move out of a static).
#[macro_export]
macro_rules! buddy_allocator_levels {
    () => {
        #[allow(unused_unsafe)]
        #[allow(static_mut_refs)]
        unsafe {
            [
                &mut $crate::vmm::allocators::kmalloc::state::LEVEL_0 as &'static mut dyn $crate::bitmap::StaticBitmap,
                &mut $crate::vmm::allocators::kmalloc::state::LEVEL_1 as &'static mut dyn $crate::bitmap::StaticBitmap,
                &mut $crate::vmm::allocators::kmalloc::state::LEVEL_2 as &'static mut dyn $crate::bitmap::StaticBitmap,
                &mut $crate::vmm::allocators::kmalloc::state::LEVEL_3 as &'static mut dyn $crate::bitmap::StaticBitmap,
                &mut $crate::vmm::allocators::kmalloc::state::LEVEL_4 as &'static mut dyn $crate::bitmap::StaticBitmap,
                &mut $crate::vmm::allocators::kmalloc::state::LEVEL_5 as &'static mut dyn $crate::bitmap::StaticBitmap,
                &mut $crate::vmm::allocators::kmalloc::state::LEVEL_6 as &'static mut dyn $crate::bitmap::StaticBitmap,
                &mut $crate::vmm::allocators::kmalloc::state::LEVEL_7 as &'static mut dyn $crate::bitmap::StaticBitmap,
                &mut $crate::vmm::allocators::kmalloc::state::LEVEL_8 as &'static mut dyn $crate::bitmap::StaticBitmap,
                &mut $crate::vmm::allocators::kmalloc::state::LEVEL_9 as &'static mut dyn $crate::bitmap::StaticBitmap,
                &mut $crate::vmm::allocators::kmalloc::state::LEVEL_10 as &'static mut dyn $crate::bitmap::StaticBitmap,
                &mut $crate::vmm::allocators::kmalloc::state::LEVEL_11 as &'static mut dyn $crate::bitmap::StaticBitmap,
                &mut $crate::vmm::allocators::kmalloc::state::LEVEL_12 as &'static mut dyn $crate::bitmap::StaticBitmap,
                &mut $crate::vmm::allocators::kmalloc::state::LEVEL_13 as &'static mut dyn $crate::bitmap::StaticBitmap,
                &mut $crate::vmm::allocators::kmalloc::state::LEVEL_14 as &'static mut dyn $crate::bitmap::StaticBitmap,
                &mut $crate::vmm::allocators::kmalloc::state::LEVEL_15 as &'static mut dyn $crate::bitmap::StaticBitmap,
                &mut $crate::vmm::allocators::kmalloc::state::LEVEL_16 as &'static mut dyn $crate::bitmap::StaticBitmap,
                &mut $crate::vmm::allocators::kmalloc::state::LEVEL_17 as &'static mut dyn $crate::bitmap::StaticBitmap,
                &mut $crate::vmm::allocators::kmalloc::state::LEVEL_18 as &'static mut dyn $crate::bitmap::StaticBitmap,
                &mut $crate::vmm::allocators::kmalloc::state::LEVEL_19 as &'static mut dyn $crate::bitmap::StaticBitmap,
                &mut $crate::vmm::allocators::kmalloc::state::LEVEL_20 as &'static mut dyn $crate::bitmap::StaticBitmap,
            ]
        }
    };
}
