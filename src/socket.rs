#![allow(clippy::expect_used)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
use alloc::vec::Vec;

use crate::arch::x86::{kernel_mutex::KernelMutex, scheduler::Permissions, vmm::process::VMA};

pub type SocketId = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketError {
    BadAddress,
    BadSocket,
}

pub struct Socket {
    pub references: usize,
    buffer: Vec<u8>,
}

impl Default for Socket {
    fn default() -> Self {
        Self::new()
    }
}

impl Socket {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buffer: Vec::new(),
            references: 1,
        }
    }

    pub fn write(&mut self, data: &[u8]) -> usize {
        self.buffer.extend_from_slice(data);
        data.len()
    }

    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        let n = core::cmp::min(buf.len(), self.buffer.len());
        for (i, b) in self.buffer.drain(..n).enumerate() {
            buf[i] = b;
        }
        n
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

pub struct SocketTable {
    sockets: Vec<Option<Socket>>,
    free_slots: Vec<SocketId>,
}

impl SocketTable {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            sockets: Vec::new(),
            free_slots: Vec::new(),
        }
    }

    pub fn create(&mut self) -> SocketId {
        if let Some(id) = self.free_slots.pop() {
            self.sockets[id] = Some(Socket::new());
            id
        } else {
            let id = self.sockets.len();
            self.sockets.push(Some(Socket::new()));
            id
        }
    }

    pub fn close(&mut self, id: SocketId) -> Result<(), SocketError> {
        let slot = self.sockets.get_mut(id).ok_or(SocketError::BadSocket)?;
        match slot {
            None => Err(SocketError::BadSocket),
            Some(sock) => {
                sock.references -= 1;
                if sock.references == 0 {
                    slot.take();
                    self.free_slots.push(id);
                }
                Ok(())
            }
        }
    }

    pub fn add_ref(&mut self, id: SocketId) -> Result<(), SocketError> {
        let socket = self.sockets.get_mut(id).and_then(|s| s.as_mut()).ok_or(SocketError::BadSocket)?;
        socket.references += 1;
        Ok(())
    }

    pub fn get_mut(&mut self, id: SocketId) -> Result<&mut Socket, SocketError> {
        self.sockets.get_mut(id).and_then(|s| s.as_mut()).ok_or(SocketError::BadSocket)
    }
}

impl Default for SocketTable {
    fn default() -> Self {
        Self::new()
    }
}

pub static SOCKETS: KernelMutex<SocketTable> = KernelMutex::new(SocketTable::new());

fn validate_user_range(vmas: &[VMA], addr: usize, len: usize, need_write: bool) -> bool {
    if len == 0 {
        return true;
    }
    let Some(end_inclusive) = addr.checked_add(len - 1) else {
        return false;
    };

    for vma in vmas {
        let Some(vma_end_inclusive) = vma.start.checked_add(vma.size).map(|e| e - 1) else {
            continue;
        };
        if vma.start > addr || end_inclusive > vma_end_inclusive {
            continue;
        }
        if need_write && !matches!(vma.permissions, Permissions::ReadWrite) {
            return false;
        }
        return true;
    }
    false
}

pub fn socket_write(id: SocketId, buf: usize, len: usize, vmas: &[VMA]) -> Result<usize, SocketError> {
    if !validate_user_range(vmas, buf, len, false) {
        return Err(SocketError::BadAddress);
    }
    // SAFETY:
    // The range was just validated as fully inside one of the current
    // process's VMAs, the current address space is loaded, and the slice is
    // only used for the duration of this call.
    let data = unsafe { core::slice::from_raw_parts(buf as *const u8, len) };

    let mut table = SOCKETS.lock().expect("socket_write | could not lock SOCKETS");
    let socket = table.get_mut(id)?;
    Ok(socket.write(data))
}

pub fn socket_read(id: SocketId, buf: usize, len: usize, vmas: &[VMA]) -> Result<usize, SocketError> {
    if !validate_user_range(vmas, buf, len, true) {
        return Err(SocketError::BadAddress);
    }
    // SAFETY:
    // Range was validated as ReadWrite and is in the current address space.
    let out = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, len) };

    let mut table = SOCKETS.lock().expect("socket_read | could not lock SOCKETS");
    let socket = table.get_mut(id)?;
    Ok(socket.read(out))
}

pub fn socket_create() -> SocketId {
    let mut table = SOCKETS.lock().expect("socket_create | could not lock SOCKETS");
    table.create()
}

pub fn socket_close(id: SocketId) -> Result<(), SocketError> {
    let mut table = SOCKETS.lock().expect("socket_close | could not lock SOCKETS");
    table.close(id)
}
