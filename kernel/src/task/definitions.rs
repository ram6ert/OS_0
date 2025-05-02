use crate::arch::RegisterStore;
#[repr(C, packed)]
pub struct Task {
    registers: RegisterStore,
    stack: u64,
}
