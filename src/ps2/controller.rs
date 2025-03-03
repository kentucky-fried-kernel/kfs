/// https://wiki.osdev.org/%228042%22_PS/2_Controller
use crate::ps2::{
    DATA_PORT,
    io::{flush_output_buffer, read, send_command, send_data, wait_for_data},
};

#[repr(u8)]
#[derive(PartialEq)]
pub enum Command {
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
pub enum Status {
    OutputFull = 0x01,
    InputFull = 0x02,
}

/// [Root System Description Pointer](https://wiki.osdev.org/RSDP).
///
/// Stores a pointer to the [Root System Description Table](https://wiki.osdev.org/RSDT).
#[repr(C, packed)]
struct Rsdp {
    signature: [u8; 8], // "RSD PTR "
    checksum: u8,
    oemid: [u8; 6],
    revision: u8,
    rsdt_address: u32,
}

/// Header used by [RSDT](https://wiki.osdev.org/RSDT) entries. The first 4 bytes
/// hold the [signature](https://wiki.osdev.org/RSDT#Defined_by_ACPI) used to
/// identify which system desciptor entry we are dealing with.
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

fn validate_checksum(ptr: *const u8, len: u32) -> bool {
    let mut sum: u8 = 0;

    for i in 0..len {
        sum = sum.wrapping_add(unsafe { *ptr.add(i as usize) });
    }

    sum == 0
}

/// Searches for the [RSDP](https://wiki.osdev.org/RSDP#Detecting_the_RSDP),
/// first in the [EBDA](https://wiki.osdev.org/Memory_Map_(x86)#Extended_BIOS_Data_Area_(EBDA)),
/// then in the main BIOS area (`0x000E0000..0x000FFFFF`).
fn get_rsdp() -> Result<*mut Rsdp, &'static str> {
    let ebda_addr: usize = unsafe { *(0x40E as *const u16) as usize } << 4;

    for loc in (ebda_addr..(ebda_addr + 0x400)).step_by(16) {
        let rsdp = unsafe { &*(loc as *const Rsdp) };

        if &rsdp.signature == b"RSD PTR " {
            return Ok(loc as *mut Rsdp);
        }
    }

    for loc in (0x000E0000..0x000FFFFF).step_by(16) {
        let rsdp = unsafe { &*(loc as *const Rsdp) };

        if &rsdp.signature == b"RSD PTR " {
            return Ok(loc as *mut Rsdp);
        }
    }

    Err("RSDP not found")
}

/// Searches for the [FADT](https://wiki.osdev.org/FADT) in the
/// [RSDT](https://wiki.osdev.org/RSDT), which we can then use to verify the
/// PS/2 configuration.
fn get_fadt(header: &SDTHeader, rsdt: *const Rsdt) -> Result<*mut Fadt, &'static str> {
    let entries = (header.length as usize - size_of::<SDTHeader>()) / 4;
    let entries_ptr = (rsdt as usize + size_of::<SDTHeader>()) as *const u32;

    for i in 0..entries {
        let entry_addr = unsafe { *entries_ptr.add(i) };
        let entry_hdr = unsafe { &*(entry_addr as *const SDTHeader) };

        if &entry_hdr.signature == b"FACP" {
            return Ok(entry_addr as *mut Fadt);
        }
    }

    Err("FADT not found")
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

/// Performs interface tests to check the PS/2 ports.
///
/// https://wiki.osdev.org/%228042%22_PS/2_Controller#Initialising_the_PS/2_Controller
fn test_port(cmd: Command) -> Result<(), &'static str> {
    assert!(cmd == Command::TestFirstPort || cmd == Command::TestSecondPort);

    send_command(cmd);
    match wait_for_data() {
        0x01 => return Err("clock line stuck low"),
        0x02 => return Err("clock line stuck high"),
        0x03 => return Err("data line stuck low"),
        0x04 => return Err("data line stuck high"),
        _ => {}
    }

    Ok(())
}

/// Determines whether a PS/2 controller is present by checking the
/// [Fixed ACPI Description Table](https://wiki.osdev.org/FADT).
///
/// https://wiki.osdev.org/%228042%22_PS/2_Controller#Initialising_the_PS/2_Controller
fn has_ps2_controller() -> Result<(), &'static str> {
    let rsdp_ptr: *mut Rsdp = get_rsdp()?;
    let rsdp: &mut Rsdp = unsafe { &mut *rsdp_ptr };
    assert_eq!(&rsdp.signature, b"RSD PTR ");
    if !validate_checksum(rsdp_ptr as *const u8, size_of::<Rsdp>() as u32) {
        return Err("RSDP checksum verification failed");
    }

    let rsdt = rsdp.rsdt_address as *const Rsdt;
    let header = unsafe { &(*rsdt).h };
    if !validate_checksum(header as *const SDTHeader as *const u8, header.length) {
        return Err("SDTHeader checksum verification failed");
    }

    let fadt_ptr = get_fadt(header, rsdt)?;
    let fadt = unsafe { &*fadt_ptr };
    if !validate_checksum(fadt_ptr as *const u8, unsafe { (*(fadt as *const Fadt as *const SDTHeader)).length }) {
        return Err("FADT checsum verification failed");
    }

    // If we are under ACPI 1.0, a PS/2 controller is assumed to be present.
    // Else if we are under 2.0+ and the "8042" flag is not set, no PS/2 controller
    // is present.
    if rsdp.revision < 2 || (fadt.boot_architecture_flags & 0x2) != 0 {
        return Ok(());
    }

    Err("no PS/2 controller found")
}

/// `AND`s the existing [config](https://wiki.osdev.org/%228042%22_PS/2_Controller#PS/2_Controller_Configuration_Byte)
/// with `new_config` and updates it.
fn update_config(new_config: u8) {
    send_command(Command::ReadConfig);
    let config = unsafe { read(DATA_PORT) };

    let new_config = config & new_config;

    send_command(Command::WriteConfig);
    send_data(new_config);
}

/// Sends `0xFF` to the PS/2 controller, resetting it, and verifies that it returns `0xAA` and `0xFA`
/// (order irrelevant), indicating success.
fn reset_controller() -> Result<(), &'static str> {
    let (mut got_0xfa, mut got_0xaa) = (false, false);
    send_data(0xFF);
    for _ in 0..2 {
        match wait_for_data() {
            0xFA => got_0xfa = true,
            0xAA => got_0xaa = true,
            _ => return Err("PS/2 controller self test failed"),
        }
    }

    if !got_0xaa || !got_0xfa {
        return Err("PS/2 controller self test failed");
    }

    Ok(())
}

/// Initializes the PS/2 controller. Note that verifying the existence of the PS/2 controller
/// is not strictly necessary, since it is assumed to be there in i386, but it was fun.
///
/// https://wiki.osdev.org/%228042%22_PS/2_Controller#Initialising_the_PS/2_Controller
/// https://wiki.osdev.org/ACPI
pub fn init() -> Result<(), &'static str> {
    has_ps2_controller()?;

    send_command(Command::DisableFirstPort);
    send_command(Command::DisableSecondPort);

    flush_output_buffer();

    update_config(0b10101110);

    send_command(Command::SelfTest);
    let test_result = wait_for_data();
    if test_result != 0x55 {
        return Err("PS/2 controller self test failed");
    }

    let is_dual_controller = is_dual_channel_controller();
    if is_dual_controller {
        update_config(0b10001100);
    }

    test_port(Command::TestFirstPort)?;

    if is_dual_controller {
        test_port(Command::TestSecondPort)?;
    }

    send_command(Command::EnableFirstPort);
    send_command(Command::EnableSecondPort);

    reset_controller()?;

    Ok(())
}
