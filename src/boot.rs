pub const STACK_SIZE: usize = 2 << 20;

#[used]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".bss")]
pub static mut STACK: Stack = Stack([0; STACK_SIZE]);

#[allow(unused)]
#[repr(align(4096))]
#[allow(static_mut_refs)]
pub struct Stack([u8; STACK_SIZE]);

impl Stack {
    pub fn as_ptr(&self) -> *const u8 {
        self.0.as_ptr()
    }
}

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
    flags: 1 | 2,
    checksum: (0usize.wrapping_sub(0x1badb002 + (1 | 2))),
};

#[used]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".data")]
static INITIAL_PAGE_DIR: [usize; 1024] = {
    let mut dir = [0usize; 1024];

    dir[0] = 0b10000011;

    dir[768] = (0 << 22) | 0b10000011;
    dir[769] = (1 << 22) | 0b10000011;
    dir[770] = (2 << 22) | 0b10000011;
    dir[771] = (3 << 22) | 0b10000011;

    dir
};

#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text")]
pub unsafe extern "C" fn higher_half() {
    core::arch::naked_asm!(
        "mov esp, offset STACK + {STACK_SIZE}",
        "push ebx",
        "push eax",
        "xor ebp, ebp",
        "call kmain",
        "halt:",
        "hlt",
        "jmp halt",
        STACK_SIZE = const STACK_SIZE
    )
}

/// # Safety
/// This function marks the entrypoint of the kernel executable.
#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".boot")]
pub unsafe extern "C" fn _start() {
    core::arch::naked_asm!(
        "mov ecx, offset INITIAL_PAGE_DIR - 0xC0000000",
        "mov cr3, ecx",
        "mov ecx, cr4",
        "or ecx, 0x10",
        "mov cr4, ecx",
        "mov ecx, cr0",
        "or ecx, 0x80000000",
        "mov cr0, ecx",
        "jmp higher_half",
    )
}
