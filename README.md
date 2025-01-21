# kfs
This is our repository for 42's `kfs` (kernel from scratch) project. The aim is to build
a fully functional 32-bit kernel. 

## Running the Kernel
`qemu` and `grub-mkrescue` are very specific about their mutual versions, so we made a [DevContainer](.devcontainer) to make it work on every Linux machine.

### Clone kfs
```sh
git clone git@github.com:kentucky-fried-kernel/kfs.git && cd kfs
```
#### VSCode
If you are in `VSCode`, you can just install the [Dev Container](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers) extension. Once installed, `VSCode` will prompt you to "Rebuild and Reopen in Container". The first time will take a while, as a whole Docker Image has to be built.

_Note: If you do not get the Dev Container prompt, you can also open the `VSCode` settings (Ctrl/Cmd+Shift+P) and search for 'Dev Container: Rebuild and Reopen in Container'._

#### Shell
If you don't have `VSCode`, you can:
```sh
# Build the Docker Image
docker build -f .devcontainer/Dockerfile -t kfs .devcontainer 
# Mount the kernel code and run it
docker run -v .:/root -it kfs bash
```
---
**If you have a window manager** you can now run `make run`, which will run our kernel in a `qemu` window.

**If you have a headless system** you can now run `make debug`, which will run the kernel in your shell.

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
- [ ] kfs-2
  - [ ] Implements Global Descriptor Table (GDT)
    - [ ] Kernel Code
    - [ ] Kernel Data
    - [ ] Kernel Stack
    - [ ] User Code
    - [ ] User Data
    - [ ] User Stack
    - [ ] GDT must be set at address 0x000008000
  - [ ] Tool to print the kernel stack
  - [ ] Basic shell with commands like `reboot`, `halt`, etc.
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
