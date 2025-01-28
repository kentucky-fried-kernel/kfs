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
        prompt_start = s.cursor;
        flush(s);

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

fn str_eq_prompt(s: &str, prompt: &[u8]) -> bool {
    for (i, c) in s.as_bytes().iter().enumerate() {
        if *c != prompt[i] {
            return false;
        }
    }

    true
}

fn promt_execute(prompt: &[u8], s: &mut Screen) {
    if str_eq_prompt("echo", prompt) {
        echo(s)
    }
}

fn add_char(p: &mut [u8; PROMPT_MAX_LENGTH], c: u8) {
    let mut i = 0;
    for (ind, ch) in p.iter_mut().enumerate() {
        if *ch == 0 {
            i = ind;
            break;
        }
    }
    p[i] = c;
}

pub fn echo(s: &mut Screen) {
    s.write_str("ECHO\n");
}
