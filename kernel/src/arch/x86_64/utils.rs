use core::arch::asm;

use crate::mm::definitions::{Frame, PageTable};

use super::{
    int::{init_8259a, init_gsbase},
    load_gdt, load_idt, logging,
    mm::page_table::PageTable as X86PageTable,
    syscall::init_syscall,
    timer::init_timer,
};

#[inline(always)]
pub fn halt() {
    unsafe {
        asm!("hlt");
    }
}

pub fn init() {
    logging::init();
    unsafe {
        load_gdt();
        load_idt();
        init_8259a();
        init_timer();
        init_nonexecutable_paging();
        init_syscall();
        init_gsbase();
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

#[inline(always)]
pub unsafe fn bind_pt_and_switch_stack(pt: &X86PageTable, stack: usize, callback: fn() -> !) -> ! {
    unsafe {
        pt.bind();
        asm!(
            "mov rsp, {}",
            in(reg) stack,
        );
    }
    callback();
}
