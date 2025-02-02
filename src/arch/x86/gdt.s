# Used in src/gdt.rs
.global flush_gdt_registers

gdtr:
	.short 0x37 # Limit
	.long 0x800 # Base

# Must be called after writing the GDT entries to the base address
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
