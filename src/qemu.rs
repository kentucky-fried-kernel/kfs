use crate::port::Port;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
#[allow(unused)]
pub enum ExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub unsafe fn exit(exit_code: ExitCode) {
    unsafe {
        let port = Port::new(0xf4);
        port.write(exit_code as u8);
    }
}
