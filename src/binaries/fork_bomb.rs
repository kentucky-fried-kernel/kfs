use core::arch::naked_asm;

use crate::binary::{Binary, Permissions, Segment};

#[unsafe(naked)]
extern "C" fn program_wait() {
    naked_asm!(
        "bomb:",
        // fork()
        "mov eax, 57",
        "int 0x80",
        "jmp bomb",
    );
}
pub fn fork_bomb_test() -> Binary {
    let mut bin = Binary::new(0x1000, 0x0000);

    // Signal handler code, mapped at a known absolute vaddr the
    // main program can hardcode when calling sys_signal.
    bin.segments.push(Segment {
        offset: Some(program_wait as *const () as usize),
        vaddr: 0x1000,
        size: 0x1000,
        permissions: Permissions::Read,
    });

    bin
}
