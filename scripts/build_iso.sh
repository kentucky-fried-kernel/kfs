#! /usr/bin/env sh

LOGLEVEL=${LOGLEVEL:-INFO}

mkdir -p ./build/iso/boot/grub

path=$1

log() {
    [ "$LOGLEVEL" == "DEBUG" ] && echo $1
}

run() {
    if [ "$LOGLEVEL" != "DEBUG" ]
    then
        $1 > /dev/null 2>&1
    else
        $1
    fi
}

cp ./grub/grub.cfg ./build/iso/boot/grub && \
log "Successfully copied GRUB config to build/iso/boot/grub"

cp $path ./build/iso/boot/kernel.bin && \
log "Successfully copied $path to ./build/iso/kernel.bin"

run "grub-mkrescue -o ./build/kernel.iso ./build/iso --locale-directory=/dev/null --fonts=ascii" && \
log "Successfully created ./build/kernel.iso from the ./build/iso directory"
