#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use crate::{
    boot::{STACK, STACK_SIZE},
    printk, printkln,
    ps2::{self, Key},
    qemu::{ExitCode, exit},
    serial_println,
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
                            if let Err(e) = self.prompt.execute(self.screen) {
                                printkln!("{}", e);
                            }
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

impl Default for Prompt {
    fn default() -> Self {
        Self {
            entries: [b' '; PROMT_SIZE],
            len: 0,
        }
    }
}

impl Prompt {
    /// Executes the command contained in the prompt buffer
    ///
    /// # Errors
    /// Returns an error if the command is not found in the
    /// available commands
    pub fn execute(&self, screen: &mut Screen) -> Result<(), &'static str> {
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
        Err("command not found - run `help` for available commands")
    }

    /// Pushes an element to the prompt
    ///
    /// # Errors
    /// Returns an error if the prompt buffer is full
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
        self.entries = [b' '; PROMT_SIZE];
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
    Command {
        name: "clear",
        func: clear_cmd,
    },
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
    Command { name: "exit", func: exit_cmd },
];

fn echo_cmd(args: &[u8], s: &mut Screen) {
    for c in args.iter() {
        s.push(Entry::new(*c));
    }
    s.push(Entry::new(b'\n'));
}

fn clear_cmd(_: &[u8], s: &mut Screen) {
    *s = Screen::default();
}
fn reboot_cmd(_: &[u8], _: &mut Screen) {
    unsafe { core::arch::asm!("out dx, al", in("dx") 0x64, in("al") 0xFEu8) };
}
fn exit_cmd(_: &[u8], _: &mut Screen) {
    serial_println!("exited");
    unsafe { exit(ExitCode::Success) };
}
#[allow(unused)]
fn help_cmd(args: &[u8], s: &mut Screen) {
    printk!("\nAvailable commands:\n\n");
    printk!("    echo:                echoes input to the console\n");
    printk!("    clear:               clears the screen\n");
    printk!("    reboot:              reboot the kernel\n");
    printk!("    prints               display the kernel stack from %esp to the top\n");
    printk!("    printsb              display the kernel stack boundaries\n");
    printk!("    help                 display this help message\n\n");
}

fn get_stack_pointer() -> u32 {
    let sp: usize;
    unsafe {
        core::arch::asm!(
            "mov {0}, esp",
            out(reg) sp,
        );
    }

    sp as u32
}
#[allow(static_mut_refs)]
fn prints_cmd(args: &[u8], s: &mut Screen) {
    printsb_cmd(args, s);
    let sp_addr = get_stack_pointer();
    let st = unsafe { (STACK.as_ptr() as usize + STACK_SIZE) as *const u8 as u32 };
    let mut row: [u8; 16];

    assert!(sp_addr <= st);

    for row_idx in (sp_addr..st).step_by(16) {
        let ptr = row_idx as *const u8;
        row = unsafe { *(ptr.cast::<[u8; 16]>()) };
        dump_row(row, ptr);
    }
}

#[allow(static_mut_refs)]
fn printsb_cmd(_args: &[u8], _s: &mut Screen) {
    printk!("ESP: {:#08x} STACK_TOP: {:#08x}\n", get_stack_pointer(), unsafe {
        (STACK.as_ptr() as usize + STACK_SIZE) as *const u8 as u32
    });
}

/// Dumps a row of 16 bytes in the following format:
///
/// ```
/// 001c503c-001c504b 20077007 72076907 6e077407 73072007  .p.r.i.n.t.s. .`
/// ^                 ^                                    ^
/// address range     hexdump                              ASCII dump
/// ```
fn dump_row(row: [u8; 16], ptr: *const u8) {
    printk!("{:08x}-{:08x} ", ptr as u32, ptr as u32 + 15);
    for word in row.chunks(4) {
        // Reminder to future self: casting to u32 prints the bytes in little-endian.
        for byte in word {
            printk!("{:02x}", byte);
        }
        printk!(" ");
    }
    for byte in row {
        printk!("{}", (if (32..127).contains(&byte) { byte } else { b'.' }) as char);
    }
    printk!("\n");
}
