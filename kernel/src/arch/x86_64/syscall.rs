use super::utils::{rdmsr, wrmsr};

pub unsafe fn init_syscall() {
    unsafe {
        let efer = rdmsr(0xC0000080);
        wrmsr(0xC0000080, efer | 1);
    }
}
