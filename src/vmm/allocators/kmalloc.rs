#![allow(unused)]

use crate::{
    bitmap::BitMap,
    printkln, serial_println,
    vmm::{
        allocators::{backend::buddy_allocator::BuddyAllocator, kmalloc::state::*},
        paging::{
            Access, Permissions,
            mmap::{MmapError, Mode, mmap},
        },
    },
};

mod state;

#[derive(Debug)]
pub enum KmallocError {
    NotEnoughMemory,
}

#[derive(Debug)]
pub enum KfreeError {
    InvalidPointer,
}

pub const BUDDY_ALLOCATOR_SIZE: usize = 1 << 29;
static mut BUDDY_ALLOCATOR: BuddyAllocator = BuddyAllocator::new(core::ptr::null(), BUDDY_ALLOCATOR_SIZE, unsafe { LEVELS });

#[allow(static_mut_refs)]
pub fn kfree(addr: *const u8) -> Result<(), KfreeError> {
    unsafe { BUDDY_ALLOCATOR.free(addr) }
}

#[allow(static_mut_refs)]
pub fn kmalloc(size: usize) -> Result<*const u8, KmallocError> {
    unsafe { BUDDY_ALLOCATOR.alloc(size).map_err(|_| KmallocError::NotEnoughMemory) }
}

#[allow(static_mut_refs)]
pub fn init() -> Result<(), KmallocError> {
    let cache_memory = mmap(None, BUDDY_ALLOCATOR_SIZE, Permissions::ReadWrite, Access::Root, Mode::Continous).map_err(|_| KmallocError::NotEnoughMemory)?;

    printkln!(
        "Initialized Buddy Allocator of size 0x{:x}, root memory: 0x{:x}",
        BUDDY_ALLOCATOR_SIZE,
        cache_memory
    );
    let mut bm = unsafe { &mut BUDDY_ALLOCATOR };

    bm.set_root(cache_memory as *const u8);

    Ok(())
}
