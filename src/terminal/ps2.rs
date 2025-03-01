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
    EnableSecondPort = 0xA8,
    ReadConfig = 0x20,
    WriteConfig = 0x60,
    SelfTest = 0xAA,
    TestFirstPort = 0xAB,
    TestSecondPort = 0xA9,
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

/// https://wiki.osdev.org/RSDT
///
/// No need for handling 2.0 since we are building for 32-bit.
#[repr(C, packed)]
struct SDTHeader {
    signature: [u8; 4],
    length: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: u32,
    creator_id: u32,
    creator_revision: u32,
}

#[repr(C, packed)]
struct RSDT {
    h: SDTHeader,
    // std_ptr: [u32; (h.length - size_of::<SDTHeader>()) / 4],
}

/// https://wiki.osdev.org/FADT
#[repr(C, packed)]
struct FADT {
    h: SDTHeader,
    firmware_ctrl: u32,
    dsdt: u32,
    reserved: u8,
    preferred_power_management_profile: u8,
    sci_interrupt: u16,
    smi_command_port: u32,
    acpi_enable: u8,
    acpi_disable: u8,
    s4_bios_req: u8,
    pstate_control: u8,
    pm1a_event_block: u32,
    pm1b_event_block: u32,
    pm1a_control_block: u32,
    pm1b_control_block: u32,
    pm2_control_block: u32,
    pm_timer_block: u32,
    gpe0_block: u32,
    gpe1_block: u32,
    pm1_event_length: u8,
    pm1_control_length: u8,
    pm2_control_length: u8,
    pm_timer_length: u8,
    gpe0_length: u8,
    gpe1_length: u8,
    gpe1_base: u8,
    cstate_control: u8,
    worst_c2_latency: u16,
    worst_c3_latency: u16,
    flush_size: u16,
    flush_stride: u16,
    duty_offset: u8,
    duty_width: u8,
    day_alarm: u8,
    month_alarm: u8,
    century: u8,

    boot_architecture_flags: u16,

    reserved_2: u8,
    flags: u32,
}

/// Have yet to find out whether we need this in 32-bit mode, currently only used
/// as a 12-byte placeholder in `FADT`.
#[allow(unused)]
#[repr(C, packed)]
struct GenericAddressStructure {
    address_space: u8,
    bit_width: u8,
    bit_offset: u8,
    access_size: u8,
    address: u64,
}

/// Searches for the Root System Description Pointer, first in the Extended BIOS Data Area,
/// then in the main BIOS area.
///
/// https://wiki.osdev.org/RSDP#Detecting_the_RSDP
/// https://wiki.osdev.org/Memory_Map_(x86)#Extended_BIOS_Data_Area_(EBDA)
fn get_rsdp() -> *mut RSDP {
    let ebda_addr: usize = unsafe { *(0x40E as *const u16) as usize } << 4;

    let target = b"RSD PTR ";

    for loc in (ebda_addr..(ebda_addr + 0x400)).step_by(16) {
        let loc_ptr = loc as *const u8;
        let mut matches = true;

        for i in 0..8 {
            if unsafe { *loc_ptr.add(i) } != target[i] {
                matches = false;
                break;
            }
        }

        if matches {
            return loc as *mut RSDP;
        }
    }

    for loc in (0x000E0000..0x000FFFFF).step_by(16) {
        let loc_ptr = loc as *const u8;
        let mut matches = true;

        for i in 0..8 {
            if unsafe { *loc_ptr.add(i) } != target[i] {
                matches = false;
                break;
            }
        }

        if matches {
            return loc as *mut RSDP;
        }
    }

    panic!("RSDP not found");
}

fn validate_table(header: &SDTHeader) -> bool {
    let ptr = header as *const SDTHeader as *const u8;
    let mut sum: u8 = 0;

    for i in 0..header.length {
        sum = sum.wrapping_add(unsafe { *ptr.add(i as usize) });
    }

    sum == 0
}

fn get_fadt(rsdt_address: u32) -> *mut FADT {
    let rsdt = rsdt_address as *const RSDT;
    let header = unsafe { &(*rsdt).h };

    if !validate_table(header) {
        panic!("STD header is not valid")
    }

    let entries = (header.length as usize - size_of::<SDTHeader>()) / 4;
    let entries_ptr = (rsdt as usize + size_of::<SDTHeader>()) as *const u32;

    for i in 0..entries {
        let entry_addr = unsafe { *entries_ptr.add(i) };
        let entry_hdr = unsafe { &*(entry_addr as *const SDTHeader) };

        if &entry_hdr.signature == b"FACP" {
            if validate_table(entry_hdr) {
                return entry_addr as *mut FADT;
            } else {
                panic!("FADT checsum invalid")
            }
        }
    }

    panic!("FADT not found")
}

/// If we are under ACPI 1.0, the PS/2 controller is assumed to be here and does not need any
/// further configuration.
/// Else if we are under 2.0+ and the "8042" flag is not set, we can assume the PS/2 controller
/// is not present.
///
/// https://wiki.osdev.org/%228042%22_PS/2_Controller#Initialising_the_PS/2_Controller
fn has_ps2_controller(fadt: &FADT, rsdp: &RSDP) -> bool {
    rsdp.revision < 2 || (fadt.boot_architecture_flags & 0x2) != 0
}

fn validate_rsdp(rsdp_ptr: *mut RSDP) -> bool {
    let mut sum: u8 = 0;

    for i in 0..size_of::<RSDP>() {
        sum = sum.wrapping_add(unsafe { *(rsdp_ptr as *const u8).add(i) });
    }

    sum == 0
}

/// Checks whether we have a dual channel PS/2 controller by trying to enable the second port
/// and reading the config. If bit 5 (port 2 enabled) is not set, we have a single channel controller,
/// since the bit should have been set after sending the `0xA8` command.
fn is_dual_channel_controller() -> bool {
    send_command(Command::EnableSecondPort);
    send_command(Command::ReadConfig);

    let config = unsafe { read(DATA_PORT) };

    config >> 5 & 1 == 1
}

/// https://wiki.osdev.org/%228042%22_PS/2_Controller#Initialising_the_PS/2_Controller
/// https://wiki.osdev.org/%228042%22_PS/2_Controller#PS/2_Controller_Configuration_Byte
/// https://wiki.osdev.org/ACPI
pub fn init() -> Result<(), &'static str> {
    let rsdp_ptr = get_rsdp();
    let rsdp: &mut RSDP = unsafe { &mut *rsdp_ptr };

    assert!(validate_rsdp(rsdp_ptr));
    assert_eq!(&rsdp.signature, b"RSD PTR ");

    let fadt_ptr = get_fadt(rsdp.rsdt_address);
    let fadt = unsafe { &*fadt_ptr };
    if !has_ps2_controller(fadt, rsdp) {
        return Err("no PS/2 controller found");
    }

    send_command(Command::DisableFirstPort);
    send_command(Command::DisableSecondPort);

    flush_output_buffer();

    send_command(Command::ReadConfig);
    let config = unsafe { read(DATA_PORT) };

    let new_config = config & 0b10101110;

    send_command(Command::WriteConfig);
    send_data(new_config);

    send_command(Command::SelfTest);
    let test_result = wait_for_data();
    if test_result != 0x55 {
        return Err("PS/2 controller self test failed");
    }

    let is_dual_controller = is_dual_channel_controller();
    if is_dual_controller {
        send_command(Command::DisableSecondPort);
        send_command(Command::ReadConfig);
        let config = unsafe { read(DATA_PORT) };
        let new_config = config & 0b10001100;
        send_command(Command::WriteConfig);
        send_data(new_config);
    }

    send_command(Command::TestFirstPort);
    match wait_for_data() {
        0x01 => return Err("clock line stuck low"),
        0x02 => return Err("clock line stuck high"),
        0x03 => return Err("data line stuck low"),
        0x04 => return Err("data line stuck high"),
        _ => {}
    }

    if is_dual_controller {
        send_command(Command::TestSecondPort);
        match wait_for_data() {
            0x01 => return Err("clock line stuck low"),
            0x02 => return Err("clock line stuck high"),
            0x03 => return Err("data line stuck low"),
            0x04 => return Err("data line stuck high"),
            _ => {}
        }
    }

    send_command(Command::EnableFirstPort);
    send_command(Command::EnableSecondPort);

    send_data(0xFF);
    for _ in 0..2 {
        match wait_for_data() {
            0xFC => return Err("PS/2 controller self test failed"),
            _ => {}
        }
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

/// Reads all data from the output buffer, flushing it. Note that this will
/// go into an endless loop if called without disabling the ports first.
fn flush_output_buffer() {
    while unsafe { read(STATUS_PORT) } & Status::OutputFull as u8 != 0 {
        unsafe { read(DATA_PORT) };
    }
}

static mut LAST_KEY: Option<u8> = None;

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

    if code == 0xF0 {
        while !is_ps2_data_available() {}
        let _ = unsafe { read(DATA_PORT) };
        unsafe { LAST_KEY = None };
        return None;
    }

    if code == 0xE0 {
        while !is_ps2_data_available() {}
        let extended_code = unsafe { read(DATA_PORT) };
        unsafe { LAST_KEY = Some(extended_code) };
        return SCANCODE_TO_KEY[extended_code as usize].1;
    }

    unsafe { LAST_KEY = Some(code) };
    SCANCODE_TO_KEY[code as usize].1
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
pub unsafe fn read(port: u16) -> u8 {
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
#[allow(unused)]
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
    LeftAlt,
    LeftShift,
    RightAlt,
    RightShift,
    LeftCtrl,
    RightCtrl,
    CapsLock,
    NumLock,
    ScrollLock,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
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
    Point = b'.',
    Space = b' ',
    Equal = b'=',
    Comma = b',',
    Backtick = b'`',
    Semicolon = b';',
    Backslash = b'\\',
    SingleQuote = b'\'',
    SquareBracketsOpen = b'[',
    SquareBracketsClosed = b']',
    Add = b'+',
    Sub = b'-',
    Mul = b'*',
    Div = b'/',
}

use Key::*;
/// Conversion table for all characters currently supported by our kernel for PS2 input.
///
/// https://wiki.osdev.org/PS/2_Keyboard#Scan_Code_Set_2
const SCANCODE_TO_KEY: [(u8, Option<Key>); 256] = [
    (0x00, None),
    (0x01, Some(F9)),
    (0x02, None),
    (0x03, Some(F5)),
    (0x04, Some(F4)),
    (0x05, Some(F1)),
    (0x06, Some(F2)),
    (0x07, Some(F12)),
    (0x08, None),
    (0x09, Some(F10)),
    (0x0A, Some(F8)),
    (0x0B, Some(F6)),
    (0x0C, Some(F8)),
    (0x0D, Some(Tab)),
    (0x0E, Some(Backtick)),
    (0x0F, None),
    (0x10, None),
    (0x11, Some(LeftAlt)),
    (0x12, Some(LeftShift)),
    (0x13, None),
    (0x14, Some(LeftCtrl)),
    (0x15, Some(Q)),
    (0x16, Some(N1)),
    (0x17, None),
    (0x18, None),
    (0x19, None),
    (0x1A, Some(Z)),
    (0x1B, Some(S)),
    (0x1C, Some(A)),
    (0x1D, Some(W)),
    (0x1E, Some(N2)),
    (0x1F, None),
    (0x20, None),
    (0x21, Some(C)),
    (0x22, Some(X)),
    (0x23, Some(D)),
    (0x24, Some(E)),
    (0x25, Some(N4)),
    (0x26, Some(N3)),
    (0x27, None),
    (0x28, None),
    (0x29, Some(Space)),
    (0x2A, Some(V)),
    (0x2B, Some(F)),
    (0x2C, Some(T)),
    (0x2D, Some(R)),
    (0x2E, Some(N5)),
    (0x2F, None),
    (0x30, None),
    (0x31, Some(N)),
    (0x32, Some(B)),
    (0x33, Some(H)),
    (0x34, Some(G)),
    (0x35, Some(Y)),
    (0x36, Some(N6)),
    (0x37, None),
    (0x38, None),
    (0x39, None),
    (0x3A, Some(M)),
    (0x3B, Some(J)),
    (0x3C, Some(U)),
    (0x3D, Some(N7)),
    (0x3E, Some(N8)),
    (0x3F, None),
    (0x40, None),
    (0x41, Some(Comma)),
    (0x42, Some(K)),
    (0x43, Some(I)),
    (0x44, Some(O)),
    (0x45, Some(N0)),
    (0x46, Some(N9)),
    (0x47, None),
    (0x48, None),
    (0x49, Some(Point)),
    (0x4A, Some(Div)),
    (0x4B, Some(L)),
    (0x4C, Some(Semicolon)),
    (0x4D, Some(P)),
    (0x4E, Some(Sub)),
    (0x4F, None),
    (0x50, None),
    (0x51, None),
    (0x52, Some(SingleQuote)),
    (0x53, None),
    (0x54, Some(SquareBracketsOpen)),
    (0x55, Some(Equal)),
    (0x56, None),
    (0x57, None),
    (0x58, Some(CapsLock)),
    (0x59, Some(RightShift)),
    (0x5A, Some(Enter)),
    (0x5B, Some(SquareBracketsClosed)),
    (0x5C, None),
    (0x5D, Some(Backslash)),
    (0x5E, None),
    (0x5F, None),
    (0x60, None),
    (0x61, None),
    (0x62, None),
    (0x63, None),
    (0x64, None),
    (0x65, None),
    (0x66, Some(Backspace)),
    (0x67, None),
    (0x68, None),
    (0x69, Some(N1)),
    (0x6A, None),
    (0x6B, Some(N4)),
    (0x6C, Some(N7)),
    (0x6D, None),
    (0x6E, None),
    (0x6F, None),
    (0x70, Some(N0)),
    (0x71, Some(Point)),
    (0x72, Some(N2)),
    (0x73, Some(N5)),
    (0x74, Some(N6)),
    (0x75, Some(N8)),
    (0x76, Some(Escape)),
    (0x77, Some(NumLock)),
    (0x78, Some(F11)),
    (0x79, Some(Add)),
    (0x7A, Some(N3)),
    (0x7B, Some(Sub)),
    (0x7C, Some(Mul)),
    (0x7D, Some(N9)),
    (0x7E, Some(ScrollLock)),
    (0x7F, None),
    (0x80, None),
    (0x81, None),
    (0x82, None),
    (0x83, Some(F7)),
    (0x84, None),
    (0x85, None),
    (0x86, None),
    (0x87, None),
    (0x88, None),
    (0x89, None),
    (0x8A, None),
    (0x8B, None),
    (0x8C, None),
    (0x8D, None),
    (0x8E, None),
    (0x8F, None),
    (0x90, None),
    (0x91, None),
    (0x92, None),
    (0x93, None),
    (0x94, None),
    (0x95, None),
    (0x96, None),
    (0x97, None),
    (0x98, None),
    (0x99, None),
    (0x9A, None),
    (0x9B, None),
    (0x9C, None),
    (0x9D, None),
    (0x9E, None),
    (0x9F, None),
    (0xA0, None),
    (0xA1, None),
    (0xA2, None),
    (0xA3, None),
    (0xA4, None),
    (0xA5, None),
    (0xA6, None),
    (0xA7, None),
    (0xA8, None),
    (0xA9, None),
    (0xAA, None),
    (0xAB, None),
    (0xAC, None),
    (0xAD, None),
    (0xAE, None),
    (0xAF, None),
    (0xB0, None),
    (0xB1, None),
    (0xB2, None),
    (0xB3, None),
    (0xB4, None),
    (0xB5, None),
    (0xB6, None),
    (0xB7, None),
    (0xB8, None),
    (0xB9, None),
    (0xBA, None),
    (0xBB, None),
    (0xBC, None),
    (0xBD, None),
    (0xBE, None),
    (0xBF, None),
    (0xC0, None),
    (0xC1, None),
    (0xC2, None),
    (0xC3, None),
    (0xC4, None),
    (0xC5, None),
    (0xC6, None),
    (0xC7, None),
    (0xC8, None),
    (0xC9, None),
    (0xCA, None),
    (0xCB, None),
    (0xCC, None),
    (0xCD, None),
    (0xCE, None),
    (0xCF, None),
    (0xD0, None),
    (0xD1, None),
    (0xD2, None),
    (0xD3, None),
    (0xD4, None),
    (0xD5, None),
    (0xD6, None),
    (0xD7, None),
    (0xD8, None),
    (0xD9, None),
    (0xDA, None),
    (0xDB, None),
    (0xDC, None),
    (0xDD, None),
    (0xDE, None),
    (0xDF, None),
    (0xE0, None),
    (0xE1, None),
    (0xE2, None),
    (0xE3, None),
    (0xE4, None),
    (0xE5, None),
    (0xE6, None),
    (0xE7, None),
    (0xE8, None),
    (0xE9, None),
    (0xEA, None),
    (0xEB, None),
    (0xEC, None),
    (0xED, None),
    (0xEE, None),
    (0xEF, None),
    (0xF0, None),
    (0xF1, None),
    (0xF2, None),
    (0xF3, None),
    (0xF4, None),
    (0xF5, None),
    (0xF6, None),
    (0xF7, None),
    (0xF8, None),
    (0xF9, None),
    (0xFA, None),
    (0xFB, None),
    (0xFC, None),
    (0xFD, None),
    (0xFE, None),
    (0xFF, None),
];
