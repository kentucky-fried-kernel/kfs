use core::{
    iter::{self},
    u32,
};

use crate::{
    arch::x86::vmm::{
        page::PAGE_SIZE,
        page_allocator::{ORDERS, PageAllocator},
        state::PAGE_ALLOCATOR,
    },
    serial_println,
};

pub fn init() -> Result<(), ()> {
    Ok(())
}
