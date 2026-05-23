use core::arch::naked_asm;

use crate::binary::{Binary, Permissions, Segment};

#[unsafe(naked)]
extern "C" fn program_stack() {
    naked_asm!(
        "mov eax, 42",
        "push eax",
        "push eax",
        //
        "pop ebx",
        //
        "mov eax, 42",
        "int 0x80",
        //
        "mov eax, 60",
        "int 0x80",
    );
}

pub fn stack_test() -> Binary {
    let mut bin = Binary::new(0x1000, 0x4000);

    bin.segments.push(Segment {
        offset: Some(program_stack as *const () as usize),
        vaddr: 0x1000,
        size: 0x1000,
        permissions: Permissions::Read,
    });

    // Data page.
    bin.segments.push(Segment {
        offset: None,
        vaddr: 0x3000,
        size: 0x1000,
        permissions: Permissions::ReadWrite,
    });

    bin
}
