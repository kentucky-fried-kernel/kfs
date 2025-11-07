use core::arch::asm;

pub struct Port {
    port: u16,
}

impl Port {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    /// # Safety
    /// This function interacts with hardware directly and can
    /// therefore not be checked by the compiler.
    pub unsafe fn write(&mut self, val: u8) {
        unsafe {
            asm!(
                "out dx, al",
                in("dx") self.port,
                in("al") val,
            );
        }
    }

    /// # Safety
    /// This function interacts with hardware directly and can
    /// therefore not be checked by the compiler.
    pub unsafe fn read(&self) -> u8 {
        let res: u8;

        unsafe {
            asm!(
                "in al, dx",
                in("dx") self.port,
                out("al") res,
            );
        }

        res
    }
}
