#![allow(dead_code)]
mod mutex;

pub use mutex::{RwLock, SpinLock, SpinLockNoIrq};
