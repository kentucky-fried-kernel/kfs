use crate::{arch::x86::interrupts::irq, cli, sti};

pub struct GlobalInterruptLock;

/// Global interrupt lock, instantiating executes `cli`, dropping executes `sti`.
impl GlobalInterruptLock {
    #[must_use]
    pub fn lock() -> Self {
        cli!();
        Self {}
    }

    /// Explicit unlock, does the same as dropping.
    pub fn unlock(self) {
        drop(self);
    }
}

impl Drop for GlobalInterruptLock {
    fn drop(&mut self) {
        sti!();
    }
}

/// IRQ lock, instantiating masks `irq`, dropping unmasks.
pub struct IRQLock {
    irq: u8,
}

impl IRQLock {
    #[must_use]
    pub fn lock(irq: u8) -> Self {
        irq::set_mask(irq);
        Self { irq }
    }

    pub fn unlock(self) {
        drop(self);
    }
}

impl Drop for IRQLock {
    fn drop(&mut self) {
        irq::clear_mask(self.irq);
    }
}
