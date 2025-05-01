use core::arch::naked_asm;

use super::{
    KERNEL_CODE_DESCRIPTOR, USER_CODE_DESCRIPTOR,
    utils::{rdmsr, wrmsr},
};

pub unsafe fn init_syscall() {
    unsafe {
        // STAR -> selector
        wrmsr(
            0xC0000081,
            ((KERNEL_CODE_DESCRIPTOR as u64) << 32) | ((USER_CODE_DESCRIPTOR as u64) << 48),
        );

        // LSTAR -> handler
        wrmsr(0xC0000082, handle_syscall as u64);

        let efer = rdmsr(0xC0000080);
        wrmsr(0xC0000080, efer | 1);
    }
}

#[naked]
unsafe extern "sysv64" fn handle_syscall() {
    unsafe {
        naked_asm!("2:", "jmp 2b");
    }
}
