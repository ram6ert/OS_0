use core::{arch::asm, intrinsics::unreachable};

use super::int::set_gsbase;

// Sysv64: Functions preserve the registers rbx, rsp, rbp, r12, r13, r14, and r15; while rax, rdi, rsi, rdx, rcx, r8, r9, r10, r11 are scratch registers.
#[repr(C, packed)]
pub struct RegisterStore {
    rax: u64,
    rbx: u64,
    rcx: u64,
    rdx: u64,
    rsi: u64,
    rdi: u64,
    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
    rbp: u64,
    rsp: u64,
    kernel_rsp: u64,
}

impl RegisterStore {
    pub const fn new(kernel_stack: u64) -> Self {
        let mut result: RegisterStore = unsafe { core::mem::zeroed() };
        result.kernel_rsp = kernel_stack;
        result
    }
}

pub unsafe fn jump_to(addr: u64) -> ! {
    unsafe {
        asm!(
            "mov r11, 0x200",
            "sysretq",
            in("rcx") addr,
        );
        unreachable();
    }
}

pub unsafe fn set_structure_base(addr: u64) {
    unsafe {
        set_gsbase(addr);
    }
}
