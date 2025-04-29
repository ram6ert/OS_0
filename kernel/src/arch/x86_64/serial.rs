use core::{arch::asm, fmt::Write};
use lazy_static::lazy_static;

use crate::sync::SpinLock;

fn in8(port: u16) -> u8 {
    let mut result: u8;
    unsafe {
        asm!("in al, dx", in("dx") port, out("al") result);
    }
    result
}

fn out8(port: u16, data: u8) {
    unsafe { asm!("out dx, al", in("dx") port, in("al") data) }
}

pub struct Serial {
    port: u16,
}

impl Serial {
    unsafe fn init(port: u16) -> Self {
        let result = Serial { port };
        // 关闭中断
        out8(port + 1, 0x00);

        // 设置访问波特率除数寄存器
        out8(port + 3, 0x80);
        // 设置波特率为 115200 / 3 = 38400
        out8(port + 0, 0x03);
        out8(port + 1, 0x00);

        // 设置访问数据寄存器，设置 8 位数据长度
        out8(port + 3, 0x03);

        // 启用并设置 FIFO
        out8(port + 2, 0xC7);

        // 配置调制调解器
        out8(port + 4, 0x0B);

        result
    }

    pub unsafe fn write_byte(&mut self, b: u8) {
        while in8(self.port + 5) & 0x20 == 0 {}
        out8(self.port, b);
    }
}

impl Write for Serial {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            unsafe {
                self.write_byte(byte);
            }
        }
        Ok(())
    }
}

lazy_static! {
    pub static ref COM1: SpinLock<Serial> = SpinLock::new(unsafe { Serial::init(0x3F8) });
}
