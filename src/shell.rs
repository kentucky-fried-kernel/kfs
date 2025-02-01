use core::arch::asm;

use crate::{
    print::u64_to_base,
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
    } else if prompt.starts_with(b"prints") {
        print_stack(s, atohex(&prompt[7..]));
    }
}

fn atohex(bytes: &[u8]) -> usize {
    let num: &[u8];
    if bytes[0] == b'0' && bytes[1] == b'x' {
        num = &bytes[2..];
    } else {
        num = &bytes;
    }

    let mut result: usize = 0;

    for byte in num {
        let digit: u8;

        if *byte >= b'0' && *byte <= b'9' {
            digit = *byte - b'0';
        } else if *byte >= b'a' && *byte <= b'f' {
            digit = *byte - b'a' + 10;
        } else if *byte >= b'A' && *byte <= b'F' {
            digit = *byte - b'A' + 10;
        } else {
            break;
        }

        result = result * 16 + digit as usize;
    }
    result
}

fn print_stack(s: &mut Screen, addr: usize) {
    let ptr: *const u8 = addr as *const u8;

    s.write_str("0x");
    write_hex(addr as u64, s);
    s.write_str("-0x");
    write_hex(addr as u64 + 16, s);
    s.write_str(": ");
    for word_idx in 0..4 {
        s.write_str("0x");

        for byte_idx in 0..4 {
            let byte = unsafe { *ptr.add((word_idx * 4) + byte_idx) };
            write_hex_byte(byte, s);
        }
        s.write_str(" ");
    }
    s.write_str("\n");
}

fn write_hex_byte(byte: u8, s: &mut Screen) {
    let high_nibble = (byte >> 4) & 0xF;
    let low_nibble = byte & 0xF;

    s.write(if high_nibble < 10 { b'0' + high_nibble } else { b'a' + (high_nibble - 10) });
    s.write(if low_nibble < 10 { b'0' + low_nibble } else { b'a' + (low_nibble - 10) });
}

fn write_hex(value: u64, s: &mut Screen) {
    let mut nibble;
    for i in (0..8).rev() {
        nibble = ((value >> (i * 4)) & 0xF) as u8;
        s.write(if nibble < 10 { b'0' + nibble } else { b'a' + (nibble - 10) });
    }
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
