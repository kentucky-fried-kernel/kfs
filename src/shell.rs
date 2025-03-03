use core::arch::asm;

use crate::{
    printk,
    ps2::{self, Key, read_if_ready},
    terminal::{Screen, vga::Buffer},
};

const PROMPT_MAX_LENGTH: usize = 1000;

/// This is a temporary fix until we have a better allocator. It is only
/// meant for use in `launch`.
#[unsafe(link_section = ".data")]
static mut PROMPT: [u8; PROMPT_MAX_LENGTH] = [0; PROMPT_MAX_LENGTH];

pub fn launch(s: &mut Screen) {
    let mut prompt_start: usize;

    loop {
        s.write_str("sh> ");
        flush(s);

        prompt_start = s.cursor;

        loop {
            if let Some(key) = ps2::read_if_ready() {
                match key {
                    Key::Enter => {
                        #[allow(static_mut_refs)]
                        unsafe {
                            PROMPT = [0; PROMPT_MAX_LENGTH];
                            s.move_cursor_to_end();
                            for (place, data) in PROMPT.iter_mut().zip(s.buffer[prompt_start..s.cursor].iter()) {
                                *place = (*data & 0xFF) as u8
                            }
                            s.handle_key(key);
                            prompt_execute(&PROMPT, s);
                        };
                        break;
                    }
                    Key::ArrowLeft | Key::Backspace => {
                        if prompt_start < s.cursor {
                            s.handle_key(key);
                        }
                    }
                    Key::Escape => {
                        reboot_cmd(&[], s);
                    }
                    _ => s.handle_key(key),
                }
                flush(s);
            }
        }
    }
}

fn flush(s: &mut Screen) {
    let b: Buffer = Buffer::from_screen(s);
    b.flush();
}

struct Command<'a> {
    name: &'a str,
    func: fn(args: &[u8], s: &mut Screen),
}

fn prompt_execute(prompt: &[u8], s: &mut Screen) {
    static COMMANDS: &[Command] = &[
        Command { name: "echo", func: echo_cmd },
        Command {
            name: "panic",
            func: panic_cmd,
        },
        Command { name: "halt", func: halt_cmd },
        Command {
            name: "reboot",
            func: reboot_cmd,
        },
        Command {
            name: "prints",
            func: prints_cmd,
        },
        Command { name: "help", func: help_cmd },
        Command {
            name: "printsb",
            func: printsb_cmd,
        },
    ];

    let cmd_end = match prompt.iter().position(|&c| c == b' ' || c == 0) {
        Some(pos) => pos,
        None => prompt.len(),
    };
    // TODO: add a way to get the total prompt length from the prompt (`prompt.len()` does not work since the prompt
    // is padded with trailing zeros).
    let prompt_len = match prompt.iter().position(|&c| c == 0) {
        Some(pos) => pos,
        None => prompt.len(),
    };

    let cmd = &prompt[..cmd_end];

    for command in COMMANDS {
        if cmd == command.name.as_bytes() {
            let args = if cmd_end < prompt_len { &prompt[cmd_end + 1..] } else { &[] };
            (command.func)(args, s);
            return;
        }
    }
    unsafe { printk!("{}: command not found\n", core::str::from_utf8_unchecked(&cmd[..cmd_end])) };
}

#[allow(unused)]
fn help_cmd(args: &[u8], s: &mut Screen) {
    unsafe {
        printk!("\nAvailable commands:\n\n");
        printk!("    echo:                echoes input to the console\n");
        printk!("    panic:               trigger a kernel panic\n");
        printk!("    halt:                halt the kernel execution\n");
        printk!("    reboot:              reboot the kernel\n");
        printk!("    prints               display the kernel stack from %esp to the top\n");
        printk!("    printsb              display the kernel stack boundaries\n");
        printk!("    help                 display this help message\n\n");
    }
}

unsafe extern "C" {
    static stack_top: u8;
}

fn get_stack_pointer() -> u32 {
    let sp: usize;
    #[cfg(not(test))]
    unsafe {
        asm!(
            "mov {0}, esp",
            out(reg) sp,
        )
    }
    #[cfg(test)]
    unsafe {
        asm!(
            "mov {0}, rsp",
            out(reg) sp,
        )
    }

    sp as u32
}

fn printsb_cmd(_args: &[u8], _s: &mut Screen) {
    unsafe { printk!("ESP: 0x{:#08x} STACK_TOP: 0x{:#08x}\n", get_stack_pointer(), &stack_top as *const u8 as u32) };
}

/// Dumps a row of 16 bytes in the following format:
///
/// ```
/// 001c503c-001c504b 20077007 72076907 6e077407 73072007  .p.r.i.n.t.s. .`
/// ^                 ^                                    ^
/// address range     hexdump                              ASCII dump
/// ```
fn dump_row(row: [u8; 16], ptr: *const u8) {
    unsafe {
        printk!("{:08x}-{:08x} ", ptr as u32, ptr as u32 + 15);
        for word in row.chunks(4) {
            // Reminder to future self: casting to u32 prints the bytes in little-endian.
            for byte in word {
                printk!("{:02x}", byte);
            }
            printk!(" ")
        }
        for byte in row {
            printk!("{}", (if !(32..127).contains(&byte) { b'.' } else { byte }) as char);
        }
        printk!("\n");
    };
}

/// Prints the stack from %esp to the stack top.
fn prints_cmd(_args: &[u8], s: &mut Screen) {
    printsb_cmd(_args, s);
    let sp_addr = get_stack_pointer();
    let st = unsafe { &stack_top as *const u8 as u32 };
    let mut row: [u8; 16];

    assert!(sp_addr <= st);

    for row_idx in (sp_addr..st).step_by(16) {
        let ptr = row_idx as *const u8;
        row = unsafe { *(ptr as *const [u8; 16]) };
        dump_row(row, ptr);
    }
}

#[allow(unused)]
fn echo_cmd(args: &[u8], s: &mut Screen) {
    let args_len = match args.iter().position(|&c| c == 0) {
        Some(pos) => pos,
        None => args.len(),
    };

    unsafe { printk!("{}\n", core::str::from_utf8_unchecked(&args[..args_len])) };
}

fn reboot_cmd(args: &[u8], s: &mut Screen) {
    while read_if_ready().is_some() {}

    unsafe { asm!("out dx, al", in("dx") 0x64, in("al") 0xFEu8) };

    halt_cmd(args, s);
}

#[allow(unused)]
fn halt_cmd(args: &[u8], s: &mut Screen) {
    unsafe { asm!("hlt") }
}

#[allow(unused)]
fn panic_cmd(args: &[u8], s: &mut Screen) {
    panic!()
}
