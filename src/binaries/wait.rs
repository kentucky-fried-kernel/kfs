use core::arch::naked_asm;

use crate::arch::x86::scheduler::{Binary, Permissions, Segment};

#[unsafe(naked)]
extern "C" fn program_wait() {
    naked_asm!(
        // fork
        "mov eax, 57",
        "int 0x80",
        // after this: eax = 0 in child, eax = child_pid in parent

        // branch on eax
        "test eax, eax",
        "jz child",
        //
        "mov eax, 98",
        "int 0x80",
        // move to be able to print
        "mov ebx, eax",
        // Print
        "mov eax, 42",
        "int 0x80",
        // Exit
        "mov eax, 60",
        "int 0x80",
        // Child
        "child:",
        // Wait
        "mov ecx, 100000000",
        "delay_loop:",
        "dec ecx",
        "jnz delay_loop",
        // ----- child path -----
        "mov eax, 60",
        "int 0x80",
    );
}
pub fn wait_test() -> Binary {
    let mut bin = Binary::new(0x1000, 0x4000);

    // Signal handler code, mapped at a known absolute vaddr the
    // main program can hardcode when calling sys_signal.
    bin.segments.push(Segment {
        offset: Some(program_wait as *const () as usize),
        vaddr: 0x1000,
        size: 0x1000,
        permissions: Permissions::Read,
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
