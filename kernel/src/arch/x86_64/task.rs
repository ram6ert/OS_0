use core::arch::asm;

use crate::arch::x86_64::{USER_CODE_DESCRIPTOR, gdt::USER_DATA_DESCRIPTOR};

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
            "mov rax, {0}",
            // ss
            "push rax",
            // rsp
            "mov rax, 0",
            "push rax",
            // rflags
            "mov rax, 0x200",
            "push rax",
            // cs
            "mov rax, {1}",
            "push rax",
            // rip
            "push {2}",
            "",
            "mov gs:0x08, rbx",
            "mov gs:0x10, rcx",
            "mov gs:0x18, rdx",
            "mov gs:0x20, rsi",
            "mov gs:0x28, rdi",
            "mov gs:0x30, r8",
            "mov gs:0x38, r9",
            "mov gs:0x40, r10",
            "mov gs:0x48, r11",
            "mov gs:0x50, r12",
            "mov gs:0x58, r13",
            "mov gs:0x60, r14",
            "mov gs:0x68, r15",
            "mov gs:0x70, rbp",
            "mov gs:0x78, rsp",
            "mov ax, {0}",
            "mov ds, ax",
            "mov es, ax",
            "swapgs",
            "iretq",
            const USER_DATA_DESCRIPTOR + 3,
            const USER_CODE_DESCRIPTOR + 3,
            in(reg) addr,
            out("rax") _,
        );
        unreachable!()
    }
}

pub unsafe fn set_structure_base(addr: u64, switch: bool) {
    unsafe {
        set_gsbase(addr);
        if switch {
            asm!("swapgs");
        }
    }
}
