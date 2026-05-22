use core::arch::naked_asm;

use crate::arch::x86::scheduler::{Binary, Permissions, Segment};

#[unsafe(naked)]
extern "C" fn program_ipc_test() {
    naked_asm!(
        "mov eax, 5",
        "int 0x80",
        // save fd in esi (callee-saved-ish for our purposes; nothing clobbers
        // it until we use it again).
        "mov esi, eax",
        //
        // --- fork() ---
        "mov eax, 57",
        "int 0x80",
        //
        //
        // branch on eax: 0 = child, nonzero = parent
        "test eax, eax",
        "jz child",
        "mov edi, eax",
        // edi is now the child_id

        // Store pid at [0x2000] so we have a stable address to pass as buf.
        "mov dword ptr [0x2000], 0x696969",
        // socket_write(fd=esi, buf=0x2000, len=4)
        "mov eax, 8",
        "mov ebx, esi",
        "mov ecx, 0x2000",
        "mov edx, 4",
        "int 0x80",
        "mov ebx, eax",
        "mov eax, 42",
        "int 0x80",
        // exit(0). ebx = 0 so the parent's exit log is unambiguous.
        "looping:",
        "jmp looping",
        // ===== CHILD =====
        "child:",
        // Busy-loop on socket_read until it returns > 0.
        // (No blocking read yet, so we spin.)
        "retry:",
        "mov eax, 7",
        "mov ebx, esi",
        "mov ecx, 0x2000",
        "mov edx, 4",
        "int 0x80",
        // eax = bytes read. If 0, retry.
        "test eax, eax",
        "jz retry",
        // Load the received value into ebx and exit, so sys_exit prints it.
        "mov ebx, [0x2000]",
        //
        "mov eax, 42",
        "int 0x80",
        //
        "mov eax, 57",
        "int 0x80",
        "mov eax, 60",
        "int 0x80",
    );
}

pub fn sockets_test() -> Binary {
    let mut bin = Binary::new(0x1000, 0x4000);

    // Signal handler code, mapped at a known absolute vaddr the
    // main program can hardcode when calling sys_signal.
    bin.segments.push(Segment {
        offset: Some(program_ipc_test as *const () as usize),
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
