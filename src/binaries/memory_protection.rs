use core::arch::naked_asm;

use crate::binary::{Binary, Permissions, Segment};

#[unsafe(naked)]
extern "C" fn program_write_in_memory() {
    naked_asm!(
        // both processes start by writing the same initial value
        "mov dword ptr [0x2000], 0xCCCC",
        //  exit()
        "mov eax, 60",
        "int 0x80",
    );
}

pub fn memory_protection_test() -> Binary {
    let mut bin = Binary::new(0x1000, 0x4000);

    bin.segments.push(Segment {
        offset: Some(program_write_in_memory as *const () as usize),
        vaddr: 0x1000,
        size: 0x1000,
        permissions: Permissions::Read,
    });

    // Data page read
    bin.segments.push(Segment {
        offset: None,
        vaddr: 0x2000,
        size: 0x1000,
        permissions: Permissions::Read,
    });

    // Data page ReadWrite
    bin.segments.push(Segment {
        offset: None,
        vaddr: 0x3000,
        size: 0x1000,
        permissions: Permissions::ReadWrite,
    });

    bin
}
