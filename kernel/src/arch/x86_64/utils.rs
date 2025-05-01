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
    }
}
