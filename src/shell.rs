use crate::{
    ps2::{self, Key},
    qemu::{ExitCode, exit},
    serial_print, serial_println,
    terminal::{
        self, Screen,
        cursor::Cursor,
        entry::Entry,
        vga::{self, Buffer},
    },
};

type Character = u8;

pub struct Shell<'a> {
    screen: &'a mut Screen,
    prompt: Prompt,
    rows_scrolled_up: usize,
}

impl<'a> Shell<'a> {
    pub fn default(screen: &'a mut Screen) -> Self {
        Self {
            screen,
            prompt: Prompt::default(),
            rows_scrolled_up: 0,
        }
    }
    pub fn launch(&mut self) {
        loop {
            self.screen.write("sh> ");
            self.flush();

            loop {
                if let Some(key) = ps2::read_if_ready() {
                    // So that on any other key it resets to the bottom
                    match key {
                        Key::ArrowUp | Key::ArrowDown => {}
                        _ => self.rows_scrolled_up = 0,
                    }
                    match key {
                        Key::Enter => {
                            self.screen.push(Entry::new(b'\n'));
                            Cursor::hide();
                            let _ = self.prompt.execute(self.screen);
                            self.prompt.clear();
                            break;
                        }
                        Key::Backspace => {
                            if self.prompt.len != 0 {
                                self.screen.remove_last();
                                self.prompt.len -= 1;
                            }
                        }
                        Key::ArrowUp => {
                            if self.screen.lines().count() > self.rows_scrolled_up + vga::BUFFER_HEIGHT {
                                self.rows_scrolled_up += 1;
                            }
                        }
                        Key::ArrowDown => {
                            if self.rows_scrolled_up > 0 {
                                self.rows_scrolled_up -= 1;
                            }
                        }
                        _ => {
                            let _ = Self::push(self, key as u8);
                        }
                    };
                    self.flush();
                }
            }
        }
    }

    fn push(&mut self, c: Character) -> Result<(), PromptPushError> {
        self.prompt.push(c)?;
        self.screen.push(Entry::new(c));
        Ok(())
    }

    pub fn flush(&mut self) {
        // This pushing and remove_last is so that the cursor which is ON the last
        // element is one after the last element
        self.screen.push(Entry::new(b' '));
        let b = Buffer::from_screen(self.screen, self.rows_scrolled_up);
        self.screen.remove_last();
        b.flush();
    }
}

// This is to validate that the prompt is never bigger than the screen size
const PROMPT_SIZE_PERCENTAGE_OF_SCREEN_SIZE: usize = 10;
const PROMT_SIZE: usize = (terminal::screen::BUFFER_SIZE * PROMPT_SIZE_PERCENTAGE_OF_SCREEN_SIZE) / 100;

#[derive(Debug)]
pub struct Prompt {
    entries: [Character; PROMT_SIZE],
    len: usize,
}

impl Prompt {
    pub fn default() -> Self {
        Self {
            entries: [b' '; PROMT_SIZE],
            len: 0,
        }
    }

    pub fn execute(&self, screen: &mut Screen) -> Result<(), ()> {
        for command in COMMANDS {
            let cmd = &self.entries[..command.name.len()];
            let args = if command.name.len() >= self.len {
                &[]
            } else {
                &self.entries[(command.name.len() + 1)..self.len]
            };
            let cmd_has_trailing_space = self.entries[command.name.len()] == b' ';

            if cmd == command.name.as_bytes() && cmd_has_trailing_space {
                (command.func)(args, screen);
                return Ok(());
            }
        }
        Ok(())
    }

    pub fn push(&mut self, c: Character) -> Result<(), PromptPushError> {
        if self.len >= self.entries.len() {
            Err(PromptPushError::PromptFull)
        } else {
            self.entries[self.len] = c;
            self.len += 1;
            Ok(())
        }
    }

    pub fn clear(&mut self) {
        self.len = 0;
        self.entries = [b' '; PROMT_SIZE]
    }
}

pub enum PromptPushError {
    PromptFull,
}
struct Command<'a> {
    name: &'a str,
    func: fn(args: &[u8], s: &mut Screen),
}
const COMMANDS: &[Command] = &[
    Command { name: "echo", func: echo_cmd },
    // Command {
    //     name: "panic",
    //     func: panic_cmd,
    // },
    // Command { name: "halt", func: halt_cmd },
    // Command {
    //     name: "reboot",
    //     func: reboot_cmd,
    // },
    // Command {
    //     name: "prints",
    //     func: prints_cmd,
    // },
    // Command { name: "help", func: help_cmd },
    // Command {
    //     name: "printsb",
    //     func: printsb_cmd,
    // },
    Command { name: "exit", func: exit_cmd },
];

fn echo_cmd(args: &[u8], s: &mut Screen) {
    for c in args.iter() {
        s.push(Entry::new(*c));
    }
    s.push(Entry::new(b'\n'));
}

// TODO: doesn't correctly exit
fn exit_cmd(_: &[u8], _: &mut Screen) {
    serial_println!("exited");
    unsafe { exit(ExitCode::Success) };
}
