use crate::{
    ps2::{self, Key},
    qemu::{ExitCode, exit},
    serial, serial_print, serial_println,
    terminal::{self, Screen, entry::Entry, screen, vga::Buffer},
};

type Character = u8;

pub struct Shell<'a> {
    screen: &'a mut Screen,
    prompt: Prompt,
}

impl<'a> Shell<'a> {
    pub fn default(screen: &'a mut Screen) -> Self {
        Self {
            screen,
            prompt: Prompt::default(),
        }
    }
    pub fn launch(&mut self) {
        loop {
            self.screen.write("sh> ");
            self.flush();

            loop {
                if let Some(key) = ps2::read_if_ready() {
                    match key {
                        Key::Enter => {
                            self.screen.push(Entry::new(b'\n'));
                            for c in self.prompt.entries.iter().take(self.prompt.len) {
                                serial_print!("{:?} ", *c as char);
                            }
                            serial_println!("");
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
                        Key::ArrowUp => {}
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
        let b = Buffer::from_screen(self.screen);
        b.flush();
    }
}

// This is to validate that the prompt is never bigger than the screen size
const PROMT_SIZE: usize = terminal::screen::BUFFER_SIZE - 0x1000;

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
