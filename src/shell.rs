use crate::terminal::{
    ps2::{self, Key},
    vga::Buffer,
    Terminal,
};

const PROMPT_MAX_LENGTH: usize = 1000;

pub fn launch(t: &mut Terminal) {
    let mut prompt: [u8; PROMPT_MAX_LENGTH] = [0; PROMPT_MAX_LENGTH];

    loop {
        prompt = [0; PROMPT_MAX_LENGTH];
        t.write_str("sh> ");
        flush(t);

        loop {
            if let Some(key) = ps2::read_if_ready() {
                t.handle_key(key);
                match key {
                    Key::Enter => {
                        promt_execute(&mut prompt, t);
                        break;
                    }
                    _ => add_char(&mut prompt, key as u8),
                }
                flush(t);
            }
        }
    }
}

fn flush(t: &mut Terminal) {
    let b: Buffer = Buffer::from_screen(t.active_screen());
    b.flush();
}

fn str_eq_prompt(s: &str, prompt: &[u8; PROMPT_MAX_LENGTH]) -> bool {
    for (i, c) in s.as_bytes().iter().enumerate() {
        if *c != prompt[i] {
            return false;
        }
    }

    true
}

fn promt_execute(prompt: &mut [u8; PROMPT_MAX_LENGTH], t: &mut Terminal) {
    if str_eq_prompt("echo", prompt) {
        echo(t)
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

pub fn echo(t: &mut Terminal) {
    for i in 0..100 {
        t.write_str("ECHO\n");
    }
}
