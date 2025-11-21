ENTRY(_start)

SECTIONS
{
	. = 0x3c000000;
	.text : ALIGN(4K)
	{
	    . = ALIGN(8);
		KEEP(*(.multiboot))
		*(.text)
	}
	.rodata : ALIGN(4K)
	{
		*(.rodata)
	}
	.data : ALIGN(4K)
	{
		*(.data)
	}
	.bss : ALIGN(4K)
	{
	    _bss_start = .;
		*(.bss)
		_bss_end = .;
	}
}
