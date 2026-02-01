use core::sync::atomic::{AtomicBool, Ordering};

use crate::{arch::x86::interrupts::irq, cli, sti};

pub struct GlobalInterruptLock;

static GLOBAL_INTERRUPT_LOCK: AtomicBool = AtomicBool::new(false);

/// Global interrupt lock, instantiating executes `cli`, dropping executes `sti`.
///
/// Attempting to instantiate a new lock while holding another one will panic.
impl GlobalInterruptLock {
    #[must_use]
    pub fn lock() -> Self {
        assert!(
            !GLOBAL_INTERRUPT_LOCK
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                .is_err(),
            "called GlobalInterruptLock::lock() while already holding another lock"
        );

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

static IRQ_LOCK: [AtomicBool; 16] = [const { AtomicBool::new(false) }; 16];

/// IRQ lock, instantiating masks `irq`, dropping unmasks.
///
/// Attempting to instantiate a new lock while holding another one will panic.
pub struct IRQLock {
    irq: u8,
}

impl IRQLock {
    #[must_use]
    pub fn lock(irq: u8) -> Self {
        assert!((0..16).contains(&irq), "irq must be in range 0..16");
        assert!(
            !IRQ_LOCK[irq as usize]
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                .is_err(),
            "called IRQLock::lock() while already holding another lock"
        );

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
