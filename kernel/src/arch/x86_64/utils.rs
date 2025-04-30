use core::arch::asm;

use super::{load_gdt, load_idt, logging};

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
    }
}
