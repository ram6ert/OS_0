use core::arch::asm;

use crate::{
    arch::{disable_irq, enable_external_irq},
    mm::definitions::{Frame, PageTable},
    trace,
};

use super::{
    int::init_8259a, load_gdt, load_idt, logging, mm::page_table::PageTable as X86PageTable,
    syscall::init_syscall, timer::init_timer,
};

pub fn init() {
    logging::init();
    trace!("Logging initialized.");
    unsafe {
        disable_irq();
        load_gdt();
        load_idt();
        init_8259a();
        init_timer();
        enable_external_irq();
    }

    trace!("Initializing misc...");
    unsafe {
        init_nonexecutable_paging();
        init_syscall();
    }
}

pub unsafe fn rdmsr(addr: u32) -> u64 {
    let low: u32;
    let high: u32;
    unsafe {
        asm!(
            "rdmsr",
            in("ecx") addr,
            out("edx") high,
            out("eax") low
        )
    }
    ((high as u64) << 32) | (low as u64)
}

#[inline(always)]
pub unsafe fn wrmsr(addr: u32, data: u64) {
    unsafe {
        asm!(
            "wrmsr",
            in("ecx") addr,
            in("edx") ((data >> 32) as u32),
            in("eax") data as u32
        )
    }
}

unsafe fn init_nonexecutable_paging() {
    unsafe {
        let efer = rdmsr(0xC0000080);
        wrmsr(0xC0000080, efer | 0x800);
    }
}

pub unsafe fn get_current_page_table_frame() -> Frame {
    let cr3: usize;
    unsafe {
        asm!(
            "mov rax, cr3",
            out("rax") cr3
        );
    }
    Frame::new((cr3 >> 12) & ((1 << 36) - 1))
}
