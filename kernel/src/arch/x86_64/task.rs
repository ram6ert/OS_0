use core::{arch::asm, mem::zeroed};

use crate::{
    arch::x86_64::{
        KERNEL_CODE_DESCRIPTOR, USER_CODE_DESCRIPTOR,
        gdt::{KERNEL_DATA_DESCRIPTOR, USER_DATA_DESCRIPTOR},
    },
    mm::definitions::KERNEL_REGION_BEGIN,
    trace,
};

use super::int::set_gsbase;

// Sysv64: Functions preserve the registers rbx, rsp, rbp, r12, r13, r14, and r15; while rax, rdi, rsi, rdx, rcx, r8, r9, r10, r11 are scratch registers.
#[repr(C)]
#[derive(Debug)]
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
    rflags: u64,
    rip: u64,
}

impl crate::task::RegisterStore for RegisterStore {
    #[inline(always)]
    extern "sysv64" fn switch_to(&self) -> ! {
        let (cs, ds) = if self.rip as usize >= KERNEL_REGION_BEGIN {
            (KERNEL_CODE_DESCRIPTOR as u64, KERNEL_DATA_DESCRIPTOR as u64)
        } else {
            (
                USER_CODE_DESCRIPTOR as u64 + 3,
                USER_DATA_DESCRIPTOR as u64 + 3,
            )
        };
        unsafe {
            asm!(
                // stack
                "push {0}",
                "push {1}",
                // rflags
                "push {2}",
                // cs, ip
                "push {3}",
                "push {4}",
                // other registers
                "mov rbx, [rax + 0x08]",
                "mov rcx, [rax + 0x10]",
                "mov rdx, [rax + 0x18]",
                "mov rsi, [rax + 0x20]",
                "mov rdi, [rax + 0x28]",
                "mov r8, [rax + 0x30]",
                "mov r9, [rax + 0x38]",
                "mov r10, [rax + 0x40]",
                "mov r11, [rax + 0x48]",
                "mov r12, [rax + 0x50]",
                "mov r13, [rax + 0x58]",
                "mov r14, [rax + 0x60]",
                "mov r15, [rax + 0x68]",
                "mov rbp, [rax + 0x70]",
                "mov rax, [rax + 0x00]",
                "iretq",
                in(reg) ds,
                in(reg) self.rsp,
                in(reg) self.rflags,
                in(reg) cs,
                in(reg) self.rip,
                in("rax") self,
            );
            unreachable!()
        }
    }

    fn ksp(&self) -> usize {
        self.kernel_rsp as usize
    }

    fn update(&mut self, pc: usize, sp: usize) {
        self.rip = pc as u64;
        self.rsp = sp as u64;
    }

    fn new(pc: usize, sp: usize, ksp: usize) -> Self {
        let mut result = unsafe { core::mem::zeroed::<Self>() };
        result.rip = pc as u64;
        result.rsp = sp as u64;
        result.kernel_rsp = ksp as u64;

        result.rflags = 0x200;
        result
    }
}

impl RegisterStore {
    pub fn update_rflags(&mut self, rflags: u64) {
        self.rflags = rflags;
    }
}

#[inline(always)]
pub unsafe fn set_structure_base(addr: u64, switch: bool) {
    unsafe {
        asm!("swapgs");
        set_gsbase(addr);
        asm!("swapgs");
        if switch {
            asm!("swapgs");
        }
    }
}
