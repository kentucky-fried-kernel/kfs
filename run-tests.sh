#!/usr/bin/env sh

mkdir -p build/iso/boot/grub

path=$(python3 testbuild.py)
path="/Users/arthur/kfs/target/i386-unknown-none/release/deps/kfs-a3c2d027e22e0133"
# path="target/i386-unknown-none/release/deps/kfs-0e0ed5f61b3dd37e"
# path="target/i386-unknown-none/release/deps/foo-54206ce26b02d34f"
# path="target/i386-unknown-none/release/deps/kfs-9e129a6664934c12"

if [ $? -eq 0 ]
then
	echo foo > /dev/null
else
	echo "Could not build tests"
	exit 1
fi

cp grub/grub.cfg build/iso/boot/grub
echo "Successfully copied grub.cfg to build/iso/boot/grub"

cp $path build/iso/boot/kernel.bin
echo "Successfully copied $path to build/iso/boot/kernel.bin"

grub-mkrescue -o build/kernel.iso build/iso --locale-directory=/dev/null --fonts=ascii
echo "Successfully created build/kernel.iso from the build/iso/ directory"

echo "Running tests in QEMU"
qemu-system-i386 -cdrom build/kernel.iso -boot d -device isa-debug-exit,iobase=0xf4,iosize=0x04 -serial stdio -display none

if [ $? -eq 33 ]
then
	exit 0
else
	exit 1
fi
