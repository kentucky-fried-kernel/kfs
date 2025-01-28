use core::arch::asm;

use crate::terminal::{
    ps2::{self, Key},
    vga::Buffer,
    Screen,
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
    } else if str_eq_prompt("halt", prompt) {
        halt();
    }
}

pub fn echo(s: &mut Screen) {
    s.write_str("ECHO\n");
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