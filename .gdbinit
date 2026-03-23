target remote localhost:1235

set disassembly-flavor intel

b kmain

c
# layout asm
# b src/main.rs:53
# c
#
# b *0xc010316e
#
# c 

# x/wx $esp
