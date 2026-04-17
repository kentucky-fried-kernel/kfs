use core::u32;

use crate::{
    arch::x86::vmm::{page::PAGE_SIZE, state::PAGE_ALLOCATOR},
    serial_println,
};

pub fn init() -> Result<(), ()> {
    unsafe {
        let v: u32 = (u32::MAX - 1) as u32;
        serial_println!("0x{:x}", v);
        #[allow(static_mut_refs)]
        let x = PAGE_ALLOCATOR.alloc(v as usize);
        crate::serial_println!("{:?}", x);
        #[allow(static_mut_refs)]
        let x = PAGE_ALLOCATOR.alloc(v as usize);
        crate::serial_println!("{:?}", x);
    }
    Ok(())
}
