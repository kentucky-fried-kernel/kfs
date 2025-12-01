#! /usr/bin/env bash

ISO=$1
shift
QEMU_ARGS="$@"

qemu-system-i386 -cdrom $ISO $QEMU_ARGS

if [ $? -eq 33 ]
then
    exit 0
else
    exit 1
fi
