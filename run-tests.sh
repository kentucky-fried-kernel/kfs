#!/usr/bin/env sh

mkdir -p build/iso/boot/grub
i386-elf-as --32 -o build/boot.o src/arch/x86/boot.s

path=$(python3 testbuild.py)

if [ $? -eq 0 ]
then
	echo foo > /dev/null
else
	echo "Could not build tests"
	exit 1
fi

cp grub/grub.cfg build/iso/boot/grub
cp $path build/iso/boot/kernel.bin
grub-mkrescue -v -o build/kernel.iso build/iso --locale-directory=/dev/null --fonts=ascii

qemu-system-i386 -cdrom build/kernel.iso -boot d -device isa-debug-exit,iobase=0xf4,iosize=0x04 -serial stdio -display none

if [ $? -eq 33 ]
then
	exit 0
else
	exit 1
fi
