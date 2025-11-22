ENTRY(_start)

HARDWARE = 0x10000;
KERNEL_BASE = 0xC0000000;

SECTIONS
{
    . = HARDWARE;

    .boot BLOCK(4K) : ALIGN(4K)
    {
        KEEP(*(.multiboot.data))
        *(.boot)
   }

    . += KERNEL_BASE;
	.text ALIGN(4K) : AT(ADDR(.text) - KERNEL_BASE)
	{
		*(.text)
	}
	.rodata ALIGN (4K) : AT (ADDR (.rodata) - KERNEL_BASE)
	{
		*(.rodata)
	}
	.data ALIGN (4K) : AT (ADDR (.data) - KERNEL_BASE)
	{
		*(.data)
	}
	.bss ALIGN (4K) : AT (ADDR (.bss) - KERNEL_BASE)
	{
	    _bss_start = .;
		*(.bss)
		_bss_end = .;
	}
}
