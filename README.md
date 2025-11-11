# kfs

This is our repository for 42's `kfs` (kernel from scratch) project. The aim is to build
a fully functional 32-bit kernel.

## Running the Kernel

### Clone kfs

```sh
git clone git@github.com:kentucky-fried-kernel/kfs.git && cd kfs
```

### Install Dependencies

#### Debian/Ubuntu

```sh
sudo apt update && \
sudo apt install \
build-essential \
grub2 \
xorriso \
qemu-system-x86 \
gcc-multilib \
mtools
# For compatibility with MacPorts, the linker used by rustc
# is named i386-elf-ld (the default MacOS ld is not compatible
# with 32-bit ELF)
cp $(which ld) /bin/i386-elf-ld
```

#### MacOS

MacOS installation may require additional steps not documented here. Please follow any error messages encountered during setup to identify missing dependencies, and it would be highly appreciated if you open an issue about it so I can fix the docs.

[Install MacPorts](https://www.macports.org/install.php)

Install `i386-elf-gcc`
```sh
sudo port install i386-elf-gcc
```

[Install GRUB](https://wiki.osdev.org/GRUB#Installing_GRUB_2_on_OS_X)

Install `QEMU`
```sh
brew install qemu
```

#### NixOS

TODO @fbruggem

---

You can now run `make run`, which will run the kernel in a `qemu` window.

## Requirements

This project is separated into 10 subprojects.

- [x] kfs-1
    - [x] Bootable via GRUB
    - [x] ASM multiboot header
    - [x] Basic kernel library
    - [x] Basic code to print stuff on the screen
    - [x] Scroll and cursor support
    - [x] I/O interface with colors support
    - [x] Handles keyboard entries
    - [x] Handles different screens with shortcuts to switch between them
- [x] kfs-2
    - [x] Implements Global Descriptor Table (GDT)
        - [x] Kernel Code
        - [x] Kernel Data
        - [x] Kernel Stack
        - [x] User Code
        - [x] User Data
        - [x] User Stack
        - [x] GDT must be set at address 0x000008000
    - [x] Tool to print the kernel stack
    - [x] Basic shell with commands like `reboot`, `halt`, etc.
- [ ] kfs-3
    - [ ] Complete memory code structure with pagination
    - [ ] Read/Write operations on memory
    - [ ] User Space and Kernel Space memory
    - [ ] Physical and Virtual memory
    - [ ] Allocators
    - [ ] Kernel Panic handling
- [ ] kfs-4
    - [ ] Hardware Interrupts
    - [ ] Software Interrupts
    - [ ] Interrup Descriptor Table (IDT)
    - [ ] Signal Handling and Scheduling
    - [ ] Global Panic Fault handling
    - [ ] Panic & System Exit commands
        - [ ] Registers cleaning
        - [ ] Stack saving
    - [ ] Base functions for syscalls
    - [ ] Different keyboard layouts
- [ ] kfs-5
    - [ ] Basic data structure for processes
    - [ ] Process interconnection (signals, sockets, etc.)
    - [ ] Process owner
    - [ ] Process rights
    - [ ] Process interruptions
    - [ ] Process memory separation
    - [ ] Multitasking
    - [ ] `mmap`
    - [ ] Link IDT and processes
    - [ ] BSS and Data sectors in the process structure
- [ ] kfs-6
    - [ ] IDE
    - [ ] Read/Write/Delete an `ext2` filesystem
    - [ ] Basic file tree (`/sys`, `/var`, `/dev`, `/proc`)
    - [ ] Multiple partitions
    - [ ] Users
- [ ] kfs-7
    - [ ] Complete syscall table
    - [ ] Complete Unix environment
    - [ ] Password protection
    - [ ] Inter-Process Communication socket
    - [ ] Unix-like filesystem hierarchy
    - [ ] Console environments
- [ ] kfs-8
    - [ ] Kernel modules
    - [ ] Loading modules at boot time
    - [ ] Functions for communication / callback between kernel and modules
    - [ ] Special memory allocator functions to create memory ring dedicated to the modules
- [ ] kfs-9
    - [ ] Complete interface to read, parse and execute ELF files
    - [ ] Syscalls to read ELF files and launch a process with them
    - [ ] Kernel module in ELF ready to be inserted at run time
    - [ ] Memory ring for the modules (built-in and run-time modules)
- [ ] kfs-x
    - [ ] Fully functional binaries
    - [ ] `libc`
    - [ ] Posix Shell
