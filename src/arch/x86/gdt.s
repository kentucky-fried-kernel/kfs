.global flush_gdt_registers

# asdkajsd
gdtr:
	.short 0x37
	.long 0x800

flush_gdt_registers:
	lgdt gdtr
	mov %cr0, %eax
	or $0x1, %eax
	mov %eax, %cr0
	jmp $0x8, $flush

flush:
	mov $0x10, %ax  
	mov %ax, %ds
	mov %ax, %es
	mov %ax, %fs
	mov %ax, %gs
	mov %ax, %ss
	ret
