ENTRY(_start)

KERNEL_BASE = 1M;

SECTIONS
{
	. = KERNEL_BASE;
	.text : ALIGN(4K)
	{
	    . = ALIGN(8);
		KEEP(*(.multiboot))
		*(.text)
	}
	.page_table : ALIGN(4K)
	{
		*(.page_table)	
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
