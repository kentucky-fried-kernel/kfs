#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Signal {
    SigInt = 0,  // Terminates | can be overwritten
    SigKill = 1, // Terminates | can't be overwritten
    SigUsr = 2,  // User defined | default -> do nothing
    SigCont = 3,
    SigStop = 4,
}

#[derive(Copy, Clone)]
pub enum Action {
    Terminate,
    Ignore,
    Continue,
    Stop,
    Handler(usize),
}

const SIGNAL_AMOUNT: usize = 5;

#[derive(Copy, Clone)]
pub struct SignalsHandlers([Action; SIGNAL_AMOUNT]);

impl SignalsHandlers {
    pub const fn new() -> Self {
        Self([
            Action::Terminate, // SigInt
            Action::Terminate, // SigKill
            Action::Ignore,    // SigUsr
            Action::Continue,  // SigCont
            Action::Stop,      // SigStop
        ])
    }

    pub fn set(&mut self, sig: Signal, action: Action) -> Result<(), ()> {
        match sig {
            Signal::SigInt | Signal::SigUsr => {
                self.0[sig as usize] = action;
                Ok(())
            }
            _ => Err(()),
        }
    }

    pub fn get(&self, sig: Signal) -> Action {
        self.0[sig as usize]
    }
}

impl TryFrom<u8> for Signal {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Signal::SigInt),
            1 => Ok(Signal::SigKill),
            2 => Ok(Signal::SigUsr),
            3 => Ok(Signal::SigCont),
            4 => Ok(Signal::SigStop),
            _ => Err(()),
        }
    }
}
