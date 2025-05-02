use core::arch::{asm, global_asm, naked_asm};

use crate::trace;

use super::{
    KERNEL_CODE_DESCRIPTOR, USER_CODE_DESCRIPTOR,
    gdt::BEFORE_USER_DESCRIPTOR,
    utils::{rdmsr, wrmsr},
};

#[inline(always)]
pub fn syscall() {
    unsafe {
        asm!("syscall");
    }
}

unsafe extern "C" {
    fn handle_syscall();
}

pub unsafe fn init_syscall() {
    unsafe {
        // STAR -> selector
        wrmsr(
            0xC0000081,
            ((KERNEL_CODE_DESCRIPTOR as u64) << 32) | ((BEFORE_USER_DESCRIPTOR as u64) << 48),
        );

        // LSTAR -> handler
        wrmsr(0xC0000082, handle_syscall as u64);

        // enable syscall
        let efer = rdmsr(0xC0000080);
        wrmsr(0xC0000080, efer | 1);
    }
}

global_asm!(
    ".macro swapgs_if_needed",
    "cmp qword ptr [rsp + 8], 8",
    "je 3f",
    "swapgs",
    "3:",
    ".endm"
);

global_asm!(
    ".macro start_syscall",
    "swapgs_if_needed",
    "mov gs:0x00, rax",
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
    "mov gs:0x80, rsp",
    ".endmacro",
    ".macro end_syscall",
    //"mov rax, gs:0x00",
    "mov rbx, gs:0x08",
    "mov rcx, gs:0x10",
    "mov rdx, gs:0x18",
    "mov rsi, gs:0x20",
    "mov rdi, gs:0x28",
    "mov r8, gs:0x30",
    "mov r9, gs:0x38",
    "mov r10, gs:0x40",
    "mov r11, gs:0x48",
    "mov r12, gs:0x50",
    "mov r13, gs:0x58",
    "mov r14, gs:0x60",
    "mov r15, gs:0x68",
    "mov rbp, gs:0x70",
    "mov rsp, gs:0x78",
    "swapgs_if_needed",
    ".endmacro",
);

global_asm!(
    r#"
    .global handle_syscall
    handle_syscall:
    start_syscall
    call {}
    end_syscall
    sysret
    "#,
    sym handle_syscall_inner
);

extern "sysv64" fn handle_syscall_inner() {
    trace!("Syscall!");
}
