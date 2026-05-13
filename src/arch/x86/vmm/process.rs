use alloc::{collections::VecDeque, vec::Vec};

use crate::arch::x86::{
    idt::InterruptRegisters,
    scheduler::{Binary, Permissions},
    vmm::addressspace::Addressspace,
};

pub type Pid = usize;
pub type OwnerId = usize;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ProcessState {
    Running,
    Ready,
    Blocked,
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
    pub ownerId: OwnerId,
}

impl Process {
    pub fn new(binary: &Binary, parent: Parent) -> Self {
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
            ownerId: 0,
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
            ownerId: 0,
        }
    }
}

pub struct ProcessTable {
    processes: Vec<Option<Process>>,
    free_slots: Vec<Pid>,
}

impl ProcessTable {
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

impl RunQueue {
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

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }
}

pub struct Scheduler {
    pub table: ProcessTable,
    pub run_queue: RunQueue,
    pub current: Option<Pid>,
}

impl Scheduler {
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

    pub fn block(&mut self, pid: Pid) {
        self.run_queue.remove(pid);
        if let Some(proc) = self.table.get_mut(pid) {
            proc.state = ProcessState::Blocked;
        }
    }

    pub fn unblock(&mut self, pid: Pid) {
        if let Some(proc) = self.table.get_mut(pid) {
            proc.state = ProcessState::Ready;
            self.run_queue.enqueue(pid);
        }
    }

    pub fn exit(&mut self, pid: Pid) {
        self.run_queue.remove(pid);
        self.table.remove(pid);
    }

    pub fn current(&mut self) -> Option<&mut Process> {
        self.table.get_mut(self.current?)
    }
}
