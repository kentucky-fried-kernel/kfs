use alloc::{collections::VecDeque, vec::Vec};

use crate::arch::x86::{idt::InterruptRegisters, vmm::addressspace::Addressspace};

pub type Pid = usize;

#[derive(Debug, PartialEq)]
pub enum ProcessState {
    Running,
    Ready,
    Blocked,
    Zombie,
}

pub struct Process {
    pub pid: Pid,
    pub state: ProcessState,
    pub addressspace: Addressspace,
    pub saved_registers: InterruptRegisters,
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
    table: ProcessTable,
    run_queue: RunQueue,
    current: Option<Pid>,
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
