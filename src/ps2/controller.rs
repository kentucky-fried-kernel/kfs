/// https://wiki.osdev.org/%228042%22_PS/2_Controller
use core::arch::asm;

use super::{Key, scancodes::SCANCODE_TO_KEY};

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
struct Rsdp {
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
struct Rsdt {
    h: SDTHeader,
    // std_ptr: [u32; (h.length - size_of::<SDTHeader>()) / 4],
}

/// https://wiki.osdev.org/FADT
#[repr(C, packed)]
struct Fadt {
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
fn get_rsdp() -> *mut Rsdp {
    let ebda_addr: usize = unsafe { *(0x40E as *const u16) as usize } << 4;

    let target = b"RSD PTR ";

    for loc in (ebda_addr..(ebda_addr + 0x400)).step_by(16) {
        let loc_ptr = loc as *const u8;
        let mut matches = true;

        for (i, _) in target.iter().enumerate() {
            if unsafe { *loc_ptr.add(i) } != target[i] {
                matches = false;
                break;
            }
        }

        if matches {
            return loc as *mut Rsdp;
        }
    }

    for loc in (0x000E0000..0x000FFFFF).step_by(16) {
        let loc_ptr = loc as *const u8;
        let mut matches = true;

        for (i, _) in target.iter().enumerate() {
            if unsafe { *loc_ptr.add(i) } != target[i] {
                matches = false;
                break;
            }
        }

        if matches {
            return loc as *mut Rsdp;
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

fn get_fadt(rsdt_address: u32) -> *mut Fadt {
    let rsdt = rsdt_address as *const Rsdt;
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
                return entry_addr as *mut Fadt;
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
fn has_ps2_controller(fadt: &Fadt, rsdp: &Rsdp) -> bool {
    rsdp.revision < 2 || (fadt.boot_architecture_flags & 0x2) != 0
}

fn validate_rsdp(rsdp_ptr: *mut Rsdp) -> bool {
    let mut sum: u8 = 0;

    for i in 0..size_of::<Rsdp>() {
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

/// Initializes the PS/2 buffer.
///
/// https://wiki.osdev.org/%228042%22_PS/2_Controller#Initialising_the_PS/2_Controller
/// https://wiki.osdev.org/%228042%22_PS/2_Controller#PS/2_Controller_Configuration_Byte
/// https://wiki.osdev.org/ACPI
pub fn init() -> Result<(), &'static str> {
    let rsdp_ptr = get_rsdp();
    let rsdp: &mut Rsdp = unsafe { &mut *rsdp_ptr };

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
        if wait_for_data() == 0xFC {
            return Err("PS/2 controller self test failed");
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
        let _ = unsafe { read(DATA_PORT) };
        unsafe { LAST_KEY = None };
        return None;
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
