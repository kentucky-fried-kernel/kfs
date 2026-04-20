use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug)]
pub struct KernelMutex<T> {
    value: UnsafeCell<T>,
    locked: AtomicBool,
}

impl<T> KernelMutex<T> {
    pub fn new(v: T) -> Self {
        Self {
            value: UnsafeCell::new(v),
            locked: AtomicBool::from(false),
        }
    }

    pub fn lock<'a>(&'a self) -> Option<Lock<'a, T>> {
        match self.locked.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst) {
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
        self.mutex.locked.store(false, Ordering::SeqCst);
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
