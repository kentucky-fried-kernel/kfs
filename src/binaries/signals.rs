use crate::binary::{Binary, Permissions, Segment};

#[unsafe(naked)]
extern "C" fn signal_handler() {
    core::arch::naked_asm!(
        // putnbr -- kernel dumps every register over serial
        "mov eax, 42",
        "int 0x80",
        // tell the kernel we're done; it restores the pre-signal regs
        "mov eax, 72",
        "int 0x80",
        // safety net: if exit_signal_handler ever returns instead of
        // restoring, don't run off into whatever happens to follow in the page
        "sh_safety:",
        "jmp sh_safety",
    );
}

#[unsafe(naked)]
extern "C" fn program_signal_test() {
    core::arch::naked_asm!(
        // fork
        "mov eax, 57",
        "int 0x80",
        "test eax, eax",
        "jz signal_child",
        // ===== parent =====
        // stash child pid in edi for the whole lifetime of the parent
        "mov edi, eax",
        "parent_outer:",
        // burn time so the child gets to run between signals
        "mov ecx, 0x05F5E100",
        "parent_delay:",
        "dec ecx",
        "jnz parent_delay",
        // sys_kill(signal=2, pid=edi)
        "mov eax, 71",
        "mov ebx, 2",
        "mov ecx, edi",
        "int 0x80",
        "jmp parent_outer",
        // ===== child =====
        "signal_child:",
        // sys_signal(signal=2, handler_vaddr=0x1000)
        "mov eax, 70",
        "mov ebx, 2",
        "mov ecx, 0x1000",
        "int 0x80",
        // distinctive register values so putnbr output is unambiguous
        "mov eax, 0xC0DEC0DE",
        "mov ebx, 0xCAFEBABE",
        "mov ecx, 0xDEADBEEF",
        "mov edx, 0xFEEDFACE",
        "mov esi, 0x12345678",
        "mov edi, 0x87654321",
        "child_loop:",
        "jmp child_loop",
    );
}

pub fn signal_print_test() -> Binary {
    let mut bin = Binary::new(0x4000, 0x4000);

    // Signal handler code, mapped at a known absolute vaddr the
    // main program can hardcode when calling sys_signal.
    bin.segments.push(Segment {
        offset: Some(signal_handler as *const () as usize),
        vaddr: 0x1000,
        size: 0x1000,
        permissions: Permissions::Read,
    });

    // Main program code, entry at 0x4000.
    bin.segments.push(Segment {
        offset: Some(program_signal_test as *const () as usize),
        vaddr: 0x4000,
        size: 0x10000000,
        permissions: Permissions::Read,
    });

    bin
}
