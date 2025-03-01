/// https://wiki.osdev.org/%228042%22_PS/2_Controller
use core::arch::asm;

const DATA_PORT: u16 = 0x60;
const STATUS_PORT: u16 = 0x64;
const COMMAND_PORT: u16 = 0x64;
const OUTPUT_BUFFER_STATUS_BIT: u8 = 1;

#[repr(u8)]
enum Command {
    DisableFirstPort = 0xAD,
    DisableSecondPort = 0xA7,
    EnableFirstPort = 0xAE,
    ReadConfig = 0x20,
    WriteConfig = 0x60,
    SelfTest = 0xAA,
}

#[repr(u8)]
enum Status {
    OutputFull = 0x01,
    InputFull = 0x02,
}

#[repr(C, packed)]
struct RSDP {
    signature: [u8; 8], // "RSD PTR "
    checksum: u8,
    oemid: [u8; 6],
    revision: u8,
    rsdt_address: u32,
}

/// https://wiki.osdev.org/RSDP#Detecting_the_RSDP
/// https://wiki.osdev.org/Memory_Map_(x86)#Extended_BIOS_Data_Area_(EBDA)
fn get_rsdp() -> *mut RSDP {
    let ebda_addr: usize = unsafe { *(0x40E as *const u32) as usize };
    let ebda_ptr = ebda_addr as *const u8;

    let target = "RSD PTR ".as_bytes();

    for loc in ((ebda_ptr as usize)..((ebda_ptr as usize) + 0x400)).step_by(0x10) {
        let loc_val: &[u8] = unsafe { core::slice::from_raw_parts(ebda_ptr.add(loc), 8) };
        if loc_val == target {
            return loc as *mut RSDP;
        }
    }

    for loc in (0x000E0000..0x000FFFFF).step_by(0x10) {
        let loc_val: &[u8] = unsafe { core::slice::from_raw_parts(loc as *const u8, 8) };
        if loc_val == target {
            return loc as *mut RSDP;
        }
    }

    panic!()
}

/// https://wiki.osdev.org/%228042%22_PS/2_Controller#Initialising_the_PS/2_Controller
/// https://wiki.osdev.org/ACPI
pub fn init() -> Result<(), &'static str> {
    let rsdp_ptr = get_rsdp();
    let rsdp: &mut RSDP = unsafe { &mut *rsdp_ptr };

    let mut checksum: u16 = 0;
    for byte in 0..size_of::<RSDP>() {
        checksum += unsafe { *(rsdp_ptr as *const u8).add(byte) as u16 };
    }
    assert_eq!(checksum & 0xFF, 0);
    rsdp.checksum = 0;

    assert_eq!(&rsdp.signature, b"RSD PTR ");
    // rspd.revision defines the ACPI version, which will always be 0 in 32-bit mode.
    assert_eq!(rsdp.revision, 0);

    send_command(Command::DisableFirstPort);
    send_command(Command::DisableSecondPort);

    flush_output_buffer();

    send_command(Command::ReadConfig);
    let config = unsafe { read(DATA_PORT) };

    let new_config = (config | 0x03) & !0x40;

    send_command(Command::WriteConfig);
    send_data(new_config);

    send_command(Command::SelfTest);
    let test_result = wait_for_data();
    if test_result != 0x55 {
        return Err("PS/2 controller failed self-test");
    }

    send_command(Command::WriteConfig);
    send_data(new_config);

    send_command(Command::EnableFirstPort);

    send_data(0xFF);

    let ack = wait_for_data();
    if ack != 0xFA {
        return Err("keyboard did not acknowledge reset command");
    }

    let keyboard_test = wait_for_data();
    if keyboard_test != 0xAA {
        return Err("Keyboard failed self-test");
    }

    send_data(0xF0);
    if wait_for_data() != 0xFA {
        return Err("Keyboard did not acknowledge scancode set command");
    }

    send_data(2);
    if wait_for_data() != 0xFA {
        return Err("Keyboard did not acknowledge scancode set value");
    }

    send_data(0xF4);
    if wait_for_data() != 0xFA {
        return Err("Keyboard did not acknowledge enable scanning command");
    }

    Ok(())
}

fn send_command(cmd: Command) {
    while unsafe { read(STATUS_PORT) } & Status::InputFull as u8 != 0 {}

    unsafe { write(COMMAND_PORT, cmd as u8) };
}

fn send_data(data: u8) {
    while unsafe { read(STATUS_PORT) } & Status::InputFull as u8 != 0 {}

    unsafe { write(DATA_PORT, data) };
}

fn wait_for_data() -> u8 {
    while unsafe { read(STATUS_PORT) } & Status::OutputFull as u8 == 0 {}

    unsafe { read(DATA_PORT) }
}

fn flush_output_buffer() {
    while unsafe { read(STATUS_PORT) } & Status::OutputFull as u8 != 0 {
        unsafe { read(DATA_PORT) };
    }
}

/// Reads from the PS2 data port if the PS2 status port is ready. Returns `Some(KeyScanCode)`
/// if the converted scancode is a supported character.
///
/// /// ### Example Usage:
/// ```
/// let mut v = Vga::new();
///
/// if let Some(c) = read_if_ready() == KeyScanCode::A {
///     v.write_char(b'a');
/// }
pub fn read_if_ready() -> Option<Key> {
    if !is_ps2_data_available() {
        return None;
    }

    let code = unsafe { read(DATA_PORT) };

    if code == 0xE0 {
        // Wait for the next byte
        while !is_ps2_data_available() {}

        let extended_code = unsafe { read(DATA_PORT) };

        // Handle extended keys
        return match extended_code {
            0x48 => Some(Key::ArrowUp),
            0x50 => Some(Key::ArrowDown),
            0x4B => Some(Key::ArrowLeft),
            0x4D => Some(Key::ArrowRight),
            _ => None,
        };
    }

    // Filter out key release codes (most have bit 7 set in scancode set 2)
    if code & 0x80 != 0 {
        return None;
    }

    SCANCODE_TO_KEY[code as usize]
}

/// Returns `true` if the PS2 input buffer has data ready to be read,
/// meaning the least significant bit of the PS2 status port is set.
fn is_ps2_data_available() -> bool {
    status() & OUTPUT_BUFFER_STATUS_BIT != 0
}

/// Reads from `STATUS_PORT` and returns the extracted value.
fn status() -> u8 {
    let res: u8;

    unsafe {
        res = read(STATUS_PORT);
    }

    res
}

/// Reads from `port` and returns the extracted value.
/// ## SAFETY:
/// `port` is assumed to be one of `STATUS_PORT` or `DATA_PORT`. Passing another value
/// to this function will result in a panic.
///
unsafe fn read(port: u16) -> u8 {
    assert!(port == DATA_PORT || port == STATUS_PORT);

    let res: u8;

    unsafe {
        asm!(
            "in al, dx",
            in("dx") port,
            out("al") res,
        );
    }

    res
}

unsafe fn write(port: u16, val: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") val,
        );
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum Key {
    Escape,
    Tab,
    Enter,
    ArrowUp,
    Backspace,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    A = b'a',
    B = b'b',
    C = b'c',
    D = b'd',
    E = b'e',
    F = b'f',
    G = b'g',
    H = b'h',
    I = b'i',
    J = b'j',
    K = b'k',
    L = b'l',
    M = b'm',
    N = b'n',
    O = b'o',
    P = b'p',
    Q = b'q',
    R = b'r',
    S = b's',
    T = b't',
    U = b'u',
    V = b'v',
    W = b'w',
    X = b'x',
    Y = b'y',
    Z = b'z',
    N0 = b'0',
    N1 = b'1',
    N2 = b'2',
    N3 = b'3',
    N4 = b'4',
    N5 = b'5',
    N6 = b'6',
    N7 = b'7',
    N8 = b'8',
    N9 = b'9',
    Dot = b'.',
    Star = b'*',
    Space = b' ',
    Minus = b'-',
    Equal = b'=',
    Slash = b'/',
    Comma = b',',
    Backtick = b'`',
    Semicolon = b';',
    Backslash = b'\\',
    SingleQuote = b'\'',
    SquareBracketsOpen = b'[',
    SquareBracketsClosed = b']',
}

use Key::*;
/// Conversion table for all characters currently supported by our kernel for PS2 input.
const SCANCODE_TO_KEY: [Option<Key>; 256] = [
    None,
    Some(Escape),
    Some(N1),
    Some(N2),
    Some(N3),
    Some(N4),
    Some(N5),
    Some(N6),
    Some(N7),
    Some(N8),
    Some(N9),
    Some(N0),
    Some(Minus),
    Some(Equal),
    Some(Backspace),
    Some(Tab),
    Some(Q),
    Some(W),
    Some(E),
    Some(R),
    Some(T),
    Some(Y),
    Some(U),
    Some(I),
    Some(O),
    Some(P),
    Some(SquareBracketsOpen),
    Some(SquareBracketsClosed),
    Some(Enter),
    None,
    Some(A),
    Some(S),
    Some(D),
    Some(F),
    Some(G),
    Some(H),
    Some(J),
    Some(K),
    Some(L),
    Some(Semicolon),
    Some(SingleQuote),
    Some(Backtick),
    None,
    Some(Backslash),
    Some(Z),
    Some(X),
    Some(C),
    Some(V),
    Some(B),
    Some(N),
    Some(M),
    Some(Comma),
    Some(Dot),
    Some(Slash),
    None,
    Some(Star),
    None,
    Some(Space),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(ArrowUp),
    None,
    None,
    Some(ArrowLeft),
    None,
    Some(ArrowRight),
    None,
    None,
    Some(ArrowDown),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
];
