use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};

pub struct SpinLockGuard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<'a, T> Deref for SpinLockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &'a Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<'a, T> DerefMut for SpinLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<'a, T> Drop for SpinLockGuard<'a, T> {
    fn drop(&mut self) {
        self.lock._lock.store(false, Ordering::Release);
    }


impl<'a, T, L: Listener> SpinLockGuard<'a, T, L: Listener> {
    fn new(lock: &'a SpinLock<T, L>) -> Self {
        Self { lock }
    }
}

pub struct SpinLock<T> {
    data: UnsafeCell<T>,
    _lock: AtomicBool,
}

unsafe impl<T> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    pub const fn new(object: T) -> Self {
        Self {
            data: UnsafeCell::new(object),
            _lock: AtomicBool::new(false),
        }
    }

    pub fn lock(&self) -> SpinLockGuard<T> {
        while self
            ._lock
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            core::hint::spin_loop();
        }
        SpinLockGuard::new(self)
    }
}
