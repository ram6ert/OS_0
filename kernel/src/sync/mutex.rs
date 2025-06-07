use core::{
    cell::UnsafeCell,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
};

use crate::arch::{disable_irq, enable_irq};

pub type SpinLock<T> = Mutex<T, Spin>;
pub type SpinLockNoIrq<T> = Mutex<T, SpinNoIrq>;

pub trait Listener {
    fn before_lock();
    fn after_unlock();
}

pub struct MutexGuard<'a, T, L: Listener> {
    lock: &'a Mutex<T, L>,
}

pub struct Mutex<T, L: Listener> {
    data: UnsafeCell<T>,
    _lock: AtomicBool,
    _phantom: PhantomData<L>,
}

pub struct Spin {}

pub struct SpinNoIrq {}

pub struct RwLock<T> {
    data: UnsafeCell<T>,
    lock: AtomicU64,
}

pub struct ReadLockGuard<'a, T> {
    lock: &'a RwLock<T>,
}

pub struct WriteLockGuard<'a, T> {
    lock: &'a RwLock<T>,
}

// MutexGuard
impl<'a, T, L: Listener> Deref for MutexGuard<'a, T, L> {
    type Target = T;

    fn deref(&self) -> &'a Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<'a, T, L: Listener> DerefMut for MutexGuard<'a, T, L> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<'a, T, L: Listener> Drop for MutexGuard<'a, T, L> {
    fn drop(&mut self) {
        self.lock._lock.store(false, Ordering::Release);
        L::after_unlock();
    }
}

impl<'a, T, L: Listener> MutexGuard<'a, T, L> {
    fn new(lock: &'a Mutex<T, L>) -> Self {
        Self { lock }
    }
}

// Mutex
unsafe impl<T, L: Listener> Sync for Mutex<T, L> {}

impl<T, L: Listener> Mutex<T, L> {
    pub const fn new(object: T) -> Self {
        Self {
            data: UnsafeCell::new(object),
            _lock: AtomicBool::new(false),
            _phantom: PhantomData {},
        }
    }

    pub fn lock(&self) -> MutexGuard<T, L> {
        L::before_lock();
        while self
            ._lock
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            core::hint::spin_loop();
        }
        MutexGuard::new(self)
    }

    pub unsafe fn get_mut(&self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }
}

// Spin
impl Listener for Spin {
    fn before_lock() {}

    fn after_unlock() {}
}

// SpinNoIrq
impl Listener for SpinNoIrq {
    fn before_lock() {
        unsafe {
            //disable_irq();
        }
    }

    fn after_unlock() {
        unsafe {
            //enable_irq();
        }
    }
}

impl<T> RwLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
            lock: AtomicU64::new(0),
        }
    }

    pub fn exclusive_access(&self) -> WriteLockGuard<T> {
        while self
            .lock
            .compare_exchange(0, u64::MAX, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            core::hint::spin_loop();
        }
        WriteLockGuard { lock: &self }
    }

    pub fn shared_access(&self) -> ReadLockGuard<T> {
        let mut origin = self.lock.load(Ordering::Acquire);
        while origin == u64::MAX
            || self
                .lock
                .compare_exchange(origin, origin + 1, Ordering::AcqRel, Ordering::Acquire)
                .is_err()
        {
            core::hint::spin_loop();
            origin = self.lock.load(Ordering::Acquire);
        }
        ReadLockGuard { lock: &self }
    }
}

impl<'a, T> Deref for WriteLockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<'a, T> DerefMut for WriteLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<'a, T> Drop for WriteLockGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.lock.store(0, Ordering::Release);
    }
}

impl<'a, T> Deref for ReadLockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<'a, T> Drop for ReadLockGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.lock.fetch_sub(1, Ordering::Release);
    }
}

unsafe impl<T> Sync for RwLock<T> {}
