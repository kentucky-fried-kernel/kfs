const STACK_SIZE: usize = 2 << 20;

#[used]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".bss")]
pub static mut STACK: Stack = Stack([0; STACK_SIZE]);

#[allow(unused)]
#[repr(align(4096))]
pub struct Stack([u8; STACK_SIZE]);

#[allow(unused)]
#[repr(align(4))]
struct MultibootHeader {
    magic: usize,
    flags: usize,
    checksum: usize,
}

#[used]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".multiboot")]
static MULTIBOOT_HEADER: MultibootHeader = MultibootHeader {
    magic: 0x1badb002,
    flags: 0,
    checksum: (0usize.wrapping_sub(0x1badb002)),
};

/// # Safety
/// This function marks the entrypoint of the kernel executable.
#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".boot")]
pub unsafe extern "C" fn _start() {
    core::arch::naked_asm!(
        "mov esp, offset STACK + {stack_size}",
        "push eax",
        "push ebx",
        "cli",
        "call kernel_main",
        "hang:",
        "cli",
        "hlt",
        "jmp hang",
        stack_size = const STACK_SIZE
    )
}
