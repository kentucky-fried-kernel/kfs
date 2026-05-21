use alloc::{collections::VecDeque, vec::Vec};

use crate::{
    arch::x86::{
        idt::InterruptRegisters,
        scheduler::{Binary, Permissions},
        vmm::addressspace::Addressspace,
    },
    signals::{Action, Signal, SignalsHandlers},
    socket::SocketId,
};

pub type Pid = usize;
pub type OwnerId = usize;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ProcessState {
    Running,
    Ready,
    Blocked,
    Waiting,
    Zombie,
}

#[derive(Clone, Copy)]
pub struct VMA {
    pub start: usize,
    pub offset: Option<usize>,
    pub size: usize,
    pub permissions: Permissions,
}

#[derive(Debug)]
pub enum Parent {
    Root,
    Pid(Pid),
}

pub struct Process {
    pub pid: Pid,
    pub state: ProcessState,
    pub addressspace: Addressspace,
    pub saved_registers: InterruptRegisters,
    pub vmas: Vec<VMA>,
    pub parent: Parent,
    pub children: Vec<Pid>,
    pub children_stopped: Vec<Pid>,
    pub owner_id: OwnerId,
    pub socket_fds: Vec<Option<SocketId>>,
    pub super_user: bool,
    pub signal_handlers: SignalsHandlers,
    pub signals_queued: Vec<Signal>,
    pub signal_handler_saved_registers: Option<InterruptRegisters>,
}

impl Process {
    #[must_use]
    pub fn new(binary: &Binary, parent: Parent, super_user: bool) -> Self {
        Self {
            pid: 0,
            state: ProcessState::Ready,
            addressspace: Addressspace::new(),
            saved_registers: InterruptRegisters::new(binary.entry as u32, binary.stack as u32),
            vmas: binary
                .segments
                .iter()
                .map(|s| VMA {
                    start: s.vaddr,
                    offset: s.offset,
                    size: s.size,
                    permissions: s.permissions,
                })
                .collect(),
            parent,
            children: Vec::new(),
            owner_id: 0,
            socket_fds: Vec::new(),
            children_stopped: Vec::new(),
            super_user,
            signal_handlers: SignalsHandlers::new(),
            signals_queued: Vec::new(),
            signal_handler_saved_registers: None,
        }
    }

    pub fn from_process(process: &mut Process) -> Self {
        Self {
            pid: 0,
            state: ProcessState::Ready,
            addressspace: process.addressspace.fork(),
            saved_registers: process.saved_registers,
            vmas: process.vmas.clone(),
            parent: Parent::Pid(process.pid),
            children: Vec::new(),
            children_stopped: Vec::new(),
            owner_id: 0,
            socket_fds: process.socket_fds.clone(),
            super_user: process.super_user,
            signal_handlers: process.signal_handlers.clone(),
            signals_queued: Vec::new(),
            signal_handler_saved_registers: process.signal_handler_saved_registers,
        }
    }

    pub fn enter_signal_handler(&mut self, entry: usize) {
        self.signal_handler_saved_registers = Some(self.saved_registers);
        self.saved_registers.eip = entry as u32;
    }

    pub fn exit_signal_handler(&mut self) -> Result<(), ()> {
        match self.signal_handler_saved_registers {
            Some(reg) => {
                self.saved_registers = reg;
                self.signal_handler_saved_registers = None;
                Ok(())
            }
            None => Err(()),
        }
    }

    pub fn install_socket(&mut self, sid: SocketId) -> usize {
        if let Some((idx, slot)) = self.socket_fds.iter_mut().enumerate().find(|(_, s)| s.is_none()) {
            *slot = Some(sid);
            idx
        } else {
            self.socket_fds.push(Some(sid));
            self.socket_fds.len() - 1
        }
    }

    #[allow(clippy::must_use_candidate)]
    pub fn resolve_socket(&self, fd: usize) -> Option<SocketId> {
        self.socket_fds.get(fd).copied().flatten()
    }

    pub fn remove_socket(&mut self, fd: usize) -> Option<SocketId> {
        self.socket_fds.get_mut(fd)?.take()
    }
}

pub struct ProcessTable {
    processes: Vec<Option<Process>>,
    free_slots: Vec<Pid>,
}

impl Default for ProcessTable {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessTable {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            processes: Vec::new(),
            free_slots: Vec::new(),
        }
    }

    pub fn insert(&mut self, mut proc: Process) -> Pid {
        if let Some(pid) = self.free_slots.pop() {
            proc.pid = pid;
            self.processes[pid] = Some(proc);
            pid
        } else {
            let pid = self.processes.len();
            proc.pid = pid;
            self.processes.push(Some(proc));
            pid
        }
    }

    pub fn remove(&mut self, pid: Pid) -> Option<Process> {
        let slot = self.processes.get_mut(pid)?;
        let proc = slot.take();
        if proc.is_some() {
            self.free_slots.push(pid);
        }
        proc
    }

    #[must_use]
    pub fn get(&self, pid: Pid) -> Option<&Process> {
        self.processes.get(pid)?.as_ref()
    }

    pub fn get_mut(&mut self, pid: Pid) -> Option<&mut Process> {
        self.processes.get_mut(pid)?.as_mut()
    }
}

pub struct RunQueue {
    queue: VecDeque<Pid>,
}

impl Default for RunQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl RunQueue {
    #[must_use]
    pub const fn new() -> Self {
        Self { queue: VecDeque::new() }
    }

    pub fn enqueue(&mut self, pid: Pid) {
        self.queue.push_back(pid);
    }

    pub fn dequeue(&mut self) -> Option<Pid> {
        self.queue.pop_front()
    }

    pub fn remove(&mut self, pid: Pid) {
        self.queue.retain(|&p| p != pid);
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.queue.len()
    }
}

pub struct Scheduler {
    pub table: ProcessTable,
    pub run_queue: RunQueue,
    pub current: Option<Pid>,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            table: ProcessTable::new(),
            run_queue: RunQueue::new(),
            current: None,
        }
    }

    pub fn spawn(&mut self, proc: Process) -> Pid {
        let pid = self.table.insert(proc);
        self.run_queue.enqueue(pid);
        pid
    }

    pub fn schedule(&mut self) -> Option<&mut Process> {
        let pid = self.run_queue.dequeue()?;
        self.run_queue.enqueue(pid); // round-robin
        self.current = Some(pid);
        self.table.get_mut(pid)
    }

    pub fn exit(&mut self, pid: Pid) {
        self.run_queue.remove(pid);
        self.table.remove(pid);
    }

    pub fn current(&mut self) -> Option<&mut Process> {
        self.table.get_mut(self.current?)
    }
}
