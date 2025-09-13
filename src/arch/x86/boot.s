.extern kernel_main

.global _start
.global stack_top

.set MB_MAGIC, 0x1BADB002
.set MB_FLAGS, 0
.set MB_CHECKSUM, (0 - (MB_MAGIC + MB_FLAGS))

.section .multiboot

	.align 4
	.long MB_MAGIC
	.long MB_FLAGS
	.long MB_CHECKSUM

.section .bss

	.align 16
	stack_bottom:
		.skip 2 << 20
	stack_top:

	.align 4096
	page_directory:
		.skip 4096
	page_table1:
		.skip 4096

.section .text

	_start:
		mov $stack_top, %esp

        push %eax
        push %ebx
		cli

		call setup_paging
		call enable_paging
		call kernel_main

		hang:
			cli
			hlt
			jmp hang
