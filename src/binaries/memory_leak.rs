use core::arch::naked_asm;

use crate::binary::{Binary, Permissions, Segment};

#[unsafe(naked)]
extern "C" fn program_leak() {
    naked_asm!(
        "mov eax, 42",
        "int 0x80",
        "mov ecx, 10",
        "leak_delay_loop:",
        //

        // fork
        "mov eax, 57",
        "int 0x80",
        "test eax, eax",
        "jz leak_child",
        "jmp leak_end",
        //
        "leak_child:",
        "mov eax, 60",
        "int 0x80",
        //
        "leak_end:",
        "dec ecx",
        "jnz leak_delay_loop",
        // after this: eax = 0 in child, eax = child_pid in parent
        "mov eax, 42",
        "int 0x80",
        // exit()
        "mov eax, 60",
        "int 0x80",
    );
}

pub fn leak_test() -> Binary {
    let mut bin = Binary::new(0x1000, 0x4000);

    // Signal handler code, mapped at a known absolute vaddr the
    // main program can hardcode when calling sys_signal.
    bin.segments.push(Segment {
        offset: Some(program_leak as *const () as usize),
        vaddr: 0x1000,
        size: 0x1000,
        permissions: Permissions::Read,
    });

    bin
}
