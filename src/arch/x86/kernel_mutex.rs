use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug)]
pub struct KernelMutex<T> {
    value: UnsafeCell<T>,
    locked: AtomicBool,
}

// Safety:
// Because we use an [AtomicBool] to check if the [KernelMutex] is already
// locked we can garantee that only one thread at a time can access
// the value inside of [UnsafeCell]
unsafe impl<T> Sync for KernelMutex<T> {}

impl<T> KernelMutex<T> {
    pub const fn new(v: T) -> Self {
        Self {
            value: UnsafeCell::new(v),
            locked: AtomicBool::from(false),
        }
    }

    pub fn lock<'a>(&'a self) -> Option<Lock<'a, T>> {
        match self.locked.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed) {
            Ok(_) => Some(Lock {
                mutex: self,
                _marker: PhantomData,
            }),
            Err(_) => None,
        }
    }
}

#[derive(Debug)]
pub struct Lock<'mutex, T> {
    mutex: &'mutex KernelMutex<T>,
    _marker: PhantomData<T>,
}

impl<T> Drop for Lock<'_, T> {
    fn drop(&mut self) {
        self.mutex.locked.store(false, Ordering::Release);
    }
}

impl<T> Deref for Lock<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // Safety
        // We garantee that this Lock will be exclusive because we only
        // give it out when the mutex is not locked.
        unsafe { &*self.mutex.value.get() }
    }
}

impl<T> DerefMut for Lock<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Safety
        // We garantee that this Lock will be exclusive because we only
        // give it out when the mutex is not locked.
        unsafe { &mut *self.mutex.value.get() }
    }
}
