ENTRY(_start)

HARDWARE = 1M;
KERNEL_BASE = 3000M;

SECTIONS
{
	. = HARDWARE;

	.multiboot : ALIGN(4K)
	{
	    . = ALIGN(8);
		KEEP(*(.multiboot))
		*(.boot)
	}

	/* . += KERNEL_BASE; */
	.text ALIGN (4K) : AT (ADDR (.text) - KERNEL_BASE)
	{
		*(.text)
	}
	.rodata ALIGN (4K) : AT (ADDR (.rodata) - KERNEL_BASE)
	{
		*(.rodata)
	}
	/* .data ALIGN (4K) : AT (ADDR (.data) - KERNEL_BASE)
	{
		*(.data)
	} */
	/* .bss ALIGN (4K) : AT (ADDR (.bss) - KERNEL_BASE)
	{
	    _bss_start = .;
		*(.bss)
		_bss_end = .;
	} */
}
