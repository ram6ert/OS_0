use core::arch::asm;

use super::{io::out8, utils::wrmsr};

const PIC_MASTER_CMD_PORT: u16 = 0x20;
const PIC_SLAVE_CMD_PORT: u16 = 0xA0;
const PIC_MASTER_DATA_PORT: u16 = 0x21;
const PIC_SLAVE_DATA_PORT: u16 = 0xA1;

#[inline(always)]
pub unsafe fn disable_irq() {
    unsafe {
        asm!("cli");
    }
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

pub unsafe fn init_gsbase() {
    unsafe {
        wrmsr(0xC0000101, 0);
        asm!("swapgs");
    }
}

#[inline(always)]
pub unsafe fn set_gsbase(base: u64) {
    unsafe {
        wrmsr(0xC0000102, base);
    }
}

pub unsafe fn init_8259a() {
    // disable irq
    unsafe {
        disable_irq();
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

    // re-enable irq
    unsafe {
        out8(PIC_MASTER_DATA_PORT, 0xff);
        out8(PIC_SLAVE_DATA_PORT, 0xff);
        enable_irq();
    }
}

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
