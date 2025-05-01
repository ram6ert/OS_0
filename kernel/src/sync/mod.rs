#![allow(dead_code)]
mod mutex;

pub use mutex::{SpinLock, SpinLockNoIrq};
