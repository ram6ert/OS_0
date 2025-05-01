use core::arch::asm;

use super::{int::init_8259a, load_gdt, load_idt, logging, timer::init_timer};

#[inline]
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
        write_msrs();
    }
}

unsafe fn rdmsr(addr: u32) -> u64 {
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

unsafe fn wrmsr(addr: u32, data: u64) {
    unsafe {
        asm!(
            "wrmsr",
            in("ecx") addr,
            in("edx") ((data >> 32) as u32),
            in("eax") data as u32
        )
    }
}

unsafe fn write_msrs() {
    // enable non-executable and syscall
    unsafe {
        let efer = rdmsr(0xC0000080);
        wrmsr(0xC0000080, efer | 0x801);
    }
}
