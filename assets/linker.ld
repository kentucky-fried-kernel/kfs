ENTRY(_start)

SECTIONS
{
	. = 1M;				/* Skip the first MegaByte of memory because addresses that are needed for hardware access leave there*/

	.text : ALIGN(4K)	/* Section for executable code - aligned by 4K bytes*/
	{
		*(.multiboot)	/* Puts the boot.s code here */
		*(.text)		/* Puts the lib.rs / kernel_code here */
	}

	.rodata : ALIGN(4K)
	{
		*(.rodata)		/* Space for READ_ONLY data - constants / string_literals*/
	}

	.data : ALIGN(4K)
	{
		*(.data)		/* Section for globals and static variables */
	}

	.bss : ALIGN(4K)
	{
		*(.bss)			/* Heap + Stack */
	}
}
