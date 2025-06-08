use core::arch::{asm, global_asm, naked_asm};

use super::{
    KERNEL_CODE_DESCRIPTOR, USER_CODE_DESCRIPTOR, USER_DATA_DESCRIPTOR,
    gdt::BEFORE_USER_DESCRIPTOR,
    utils::{rdmsr, wrmsr},
};

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

        // IA32_FMASK
        wrmsr(0xC0000084, 0x47700u64);

        // enable syscall
        let efer = rdmsr(0xC0000080);
        wrmsr(0xC0000080, efer | 1);
    }
}

global_asm!(
    ".macro start_syscall",
    "swapgs",
    "mov gs:0x98, rsp",
    "mov rsp, gs:0x80",
    "push r9",
    "push r8",
    "push r10",
    "push rdx",
    "push rsi",
    "push rdi",
    "push rax",
    "mov rdi, rsp",
    "push rbx",
    "push rcx",
    "push r11",
    "push r12",
    "push r13",
    "push r14",
    "push r15",
    "sti",
    ".endmacro",
    ".macro end_syscall",
    "cli",
    "pop r15",
    "pop r14",
    "pop r13",
    "pop r12",
    "pop r11",
    "pop rcx",
    "pop rbx",
    "add rsp, 8",
    "pop rdi",
    "pop rsi",
    "pop rdx",
    "pop r10",
    "pop r8",
    "pop r9",
    "mov rsp, gs:0x98",
    "swapgs",
    "sysretq",
    ".endmacro"
);

global_asm!(
    r#"
    .global handle_syscall
    handle_syscall:
    start_syscall
    call {}
    end_syscall
    "#,
    sym handle_syscall_inner
);

extern "sysv64" fn handle_syscall_inner(parameters: &[u64; 7]) -> usize {
    unsafe {
        crate::user::handle_syscall(
            parameters[0] as usize,
            core::slice::from_raw_parts((&parameters[1..]).as_ptr() as *const usize, 6),
        )
    }
}
