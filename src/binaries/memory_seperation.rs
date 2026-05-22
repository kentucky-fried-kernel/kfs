use core::arch::naked_asm;

use crate::arch::x86::scheduler::{Binary, Permissions, Segment};

#[unsafe(naked)]
extern "C" fn program_write_in_memory() {
    naked_asm!(
        // both processes start by writing the same initial value
        "mov dword ptr [0x2000], 0xCCCC",
        // fork
        "mov eax, 57",
        "int 0x80",
        // after this: eax = 0 in child, eax = child_pid in parent
        "mov ebx, [0x2000]",
        "mov eax, 42",
        "int 0x80",
        // branch on eax
        "test eax, eax",
        "jz child",
        // ----- parent path -----
        // overwrite with 0xAAAA - this should NOT affect the child
        "mov dword ptr [0x2000], 0xAAAA",
        "jmp done",
        "child:",
        // ----- child path -----
        // overwrite with 0xBBBB - this should NOT affect the parent
        "mov dword ptr [0x2000], 0xBBBB",
        "done:",
        "mov ebx, [0x2000]",
        "mov eax, 42",
        "int 0x80",
        // both processes read their own [0x2000] into ebx and exit
        "mov eax, 60",
        "int 0x80",
    );
}

pub fn memory_seperation_test() -> Binary {
    let mut bin = Binary::new(0x1000, 0x4000);

    bin.segments.push(Segment {
        offset: Some(program_write_in_memory as *const () as usize),
        vaddr: 0x1000,
        size: 0x1000,
        permissions: Permissions::Read,
    });

    // Data page.
    bin.segments.push(Segment {
        offset: None,
        vaddr: 0x2000,
        size: 0x1000,
        permissions: Permissions::ReadWrite,
    });

    // Stack page.
    bin.segments.push(Segment {
        offset: None,
        vaddr: 0x3000,
        size: 0x1000,
        permissions: Permissions::ReadWrite,
    });

    bin
}
