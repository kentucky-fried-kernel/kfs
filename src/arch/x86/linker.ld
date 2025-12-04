ENTRY(_start)

SECTIONS
{
    . = 1M;

    .multiboot ALIGN(4K): {
        KEEP(*(.multiboot))
    }

    .note.gnu.build-id ALIGN(4K): {
        *(.note.gnu.build-id)
    }

    .boot ALIGN(4K): {
        *(.boot)
    }

    . += 0xC0000000;

    text_start = .;

    .text ALIGN(4K): AT(ADDR(.text) - 0xC0000000) {
        *(.text)
    }

    .rodata ALIGN(4K): AT(ADDR(.rodata) - 0xC0000000) {
        *(.rodata)
    }

    data_start = .;

    .data ALIGN(4K): AT(ADDR(.data) - 0xC0000000) {
        *(.data)
    }

    .bss ALIGN(4K): AT(ADDR(.bss) - 0xC0000000) {
        _bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        . = ALIGN(4K);
        _bss_end = .;
    }

    _kernel_end = .;
}
