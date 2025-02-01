use core::arch::asm;

use crate::{
    conv::hextou,
    terminal::{
        ps2::{self, read_if_ready, Key},
        vga::Buffer,
        Screen,
    },
};

const PROMPT_MAX_LENGTH: usize = 1000;

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
                        let mut prompt: [u8; PROMPT_MAX_LENGTH] = [0; PROMPT_MAX_LENGTH];
                        s.move_cursor_to_end();
                        for (place, data) in prompt.iter_mut().zip(s.buffer[prompt_start..s.cursor].iter()) {
                            *place = (*data & 0xFF) as u8
                        }
                        s.handle_key(key);
                        promt_execute(&prompt, s);
                        break;
                    }
                    Key::ArrowLeft | Key::Backspace => {
                        if prompt_start < s.cursor {
                            s.handle_key(key);
                        }
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

fn promt_execute(prompt: &[u8], s: &mut Screen) {
    if str_eq_prompt("echo", prompt) {
        echo(s)
    } else if str_eq_prompt("panic", prompt) {
        panic!()
    } else if str_eq_prompt("halt", prompt) {
        halt();
    } else if str_eq_prompt("reboot", prompt) {
        reboot();
    } else if prompt.starts_with(b"prints ") {
        print_stack_slice(s, &prompt[7..]);
    } else if str_eq_prompt("prints", prompt) {
        print_stack(s)
    } else if str_eq_prompt("help", prompt){
        help(s);
    } else {
        s.write_str("command not found\n");
    }
}

fn help(s: &mut Screen) {
    s.write_str("Available commands:\n\n");
    s.write_str("    echo:                print 'ECHO' to the console\n");
    s.write_str("    panic:               trigger a kernel panic\n");
    s.write_str("    halt:                halt the CPU execution\n");
    s.write_str("    reboot:              reboot the CPU\n");
    s.write_str("    prints <address>:    display 16 bytes of the stack starting from <address>\n");
    s.write_str("    prints               display the whole kernel stack\n");
    s.write_str("    help                 display this help message\n\n");
}

fn contains_non_null(bytes: &[u8]) -> bool {
    for byte in bytes {
        if *byte != 0 {
            return true;
        }
    }
    false
}

fn print_stack(s: &mut Screen) {
    let addr = 0x0;
    let ptr: *const u8 = addr as *const u8;
    let stack_size = 4096;

    for row_idx in (0..stack_size).step_by(16) {
        let mut bytes: [u8; 16] = [0u8; 16];

        for byte_idx in 0..16 {
            let byte = unsafe { *ptr.add(row_idx + byte_idx) };
            bytes[byte_idx] = byte;
        }

        if contains_non_null(&bytes) {
            s.write_str("0x");
            s.write_hex((addr + row_idx) as u32);
            s.write_str("-0x");
            s.write_hex((addr + row_idx + 15) as u32);
            s.write_str(": ");

            for byte in bytes.chunks(4) {
                s.write_str("0x");
                for b in byte {
                    s.write_hex_byte(*b);
                }
                s.write_str(" ");
            }
            s.write_str("\n");
            flush(s);
        }
    }

    s.write_str("\nStack displayed by rows of 16 bytes. Omitted rows containing only zeros.\n");
}

fn print_stack_slice(s: &mut Screen, prompt: &[u8]) {
    let addr = match hextou(prompt) {
        Some(a) => a,
        None => {
            s.write_str("No valid hex found in input\n");
            return;
        }
    };
    let ptr: *const u8 = addr as *const u8;

    s.write_str("0x");
    s.write_hex(addr as u32);
    s.write_str("-0x");
    s.write_hex(addr as u32 + 12);
    s.write_str(": ");
    for word_idx in 0..4 {
        s.write_str("0x");

        for byte_idx in 0..4 {
            let byte = unsafe { *ptr.add((word_idx * 4) + byte_idx) };
            s.write_hex_byte(byte);
        }
        s.write_str(" ");
    }
    s.write_str("\n");
}

pub fn echo(s: &mut Screen) {
    s.write_str("ECHO\n");
}

fn reboot() {
    while read_if_ready().is_some() {}

    unsafe { asm!("out dx, al", in("dx") 0x64, in("al") 0xFEu8) };

    halt();
}

fn halt() {
    unsafe { asm!("hlt") }
}

fn str_eq_prompt(s: &str, prompt: &[u8]) -> bool {
    for (i, c) in s.as_bytes().iter().enumerate() {
        if *c != prompt[i] {
            return false;
        }
    }

    true
}
