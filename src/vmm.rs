pub mod address_space;
pub mod boot;
pub mod page;
pub mod page_allocator;

pub const MEMORY_MAX: u64 = 1 << 32;

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

struct NullAllocator;

unsafe impl GlobalAlloc for NullAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut() // always fail
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // no-op
    }
}

#[global_allocator]
static GLOBAL: NullAllocator = NullAllocator;
