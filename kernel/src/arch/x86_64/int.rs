use core::arch::asm;

use crate::trace;

use super::{io::out8, utils::wrmsr};

const PIC_MASTER_CMD_PORT: u16 = 0x20;
const PIC_SLAVE_CMD_PORT: u16 = 0xA0;
const PIC_MASTER_DATA_PORT: u16 = 0x21;
const PIC_SLAVE_DATA_PORT: u16 = 0xA1;

#[inline(always)]
pub unsafe fn disable_irq() -> bool {
    let result = get_irq_enabled();
    unsafe {
        asm!("cli");
    }
    result
}

#[inline(always)]
pub unsafe fn enable_irq() {
    unsafe {
        asm!("sti");
    }
}

#[inline(always)]
pub unsafe fn enable_external_irq() {
    unsafe {
        out8(PIC_MASTER_DATA_PORT, 0xfa);
        out8(PIC_SLAVE_DATA_PORT, 0xff);
    }
}

#[inline(always)]
pub unsafe fn set_gsbase(base: u64) {
    unsafe {
        wrmsr(0xC0000102, base);
    }
}

pub unsafe fn init_8259a() {
    trace!("Initializing PIC...");
    unsafe {
        out8(PIC_MASTER_DATA_PORT, 0xff);
        out8(PIC_SLAVE_DATA_PORT, 0xff);
    }

    unsafe {
        out8(PIC_MASTER_CMD_PORT, 0x11);
        out8(PIC_MASTER_DATA_PORT, 0x20);
        out8(PIC_MASTER_DATA_PORT, 0x04);
        out8(PIC_MASTER_DATA_PORT, 0x01);

        out8(PIC_SLAVE_CMD_PORT, 0x11);
        out8(PIC_SLAVE_DATA_PORT, 0x28);
        out8(PIC_SLAVE_DATA_PORT, 0x02);
        out8(PIC_SLAVE_DATA_PORT, 0x01);
    }

    unsafe {
        out8(PIC_MASTER_DATA_PORT, 0xff);
        out8(PIC_SLAVE_DATA_PORT, 0xff);
    }
}

#[inline(always)]
pub unsafe fn send_eoi(idx: u8) {
    if idx < 8 {
        unsafe {
            out8(PIC_MASTER_CMD_PORT, 0x20);
        }
    } else {
        unsafe {
            out8(PIC_SLAVE_CMD_PORT, 0x20);
            out8(PIC_MASTER_CMD_PORT, 0x20);
        }
    }
}

#[inline(always)]
pub fn get_irq_enabled() -> bool {
    let mut result: usize = 0;
    unsafe {
        asm!("pushfq", "pop rax", inout("rax") result);
    }
    result & 0x200 != 0
}
