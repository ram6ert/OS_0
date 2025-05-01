use core::fmt::Write;

use crate::sync::SpinLock;

use super::io::{in8, out8};

pub struct Serial {
    pub port: u16,
}

impl Serial {
    unsafe fn init(port: u16) {
        unsafe {
            // disable interruption
            out8(port + 1, 0x00);

            // baudrate register
            out8(port + 3, 0x80);
            // baudrate 115200 / 3 = 38400
            out8(port + 0, 0x03);
            out8(port + 1, 0x00);

            // data register
            out8(port + 3, 0x03);

            // FIFO
            out8(port + 2, 0xC7);

            // modem
            out8(port + 4, 0x0B);
        }
    }

    pub unsafe fn write_byte(&mut self, b: u8) {
        unsafe {
            while in8(self.port + 5) & 0x20 == 0 {}
            out8(self.port, b);
        }
    }

    const fn new(port: u16) -> Self {
        Self { port }
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

pub struct SyncSerial {
    serial: SpinLock<Serial>,
}

impl Write for SyncSerial {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.serial.lock().write_str(s)
    }
}

impl SyncSerial {
    const fn new(port: u16) -> Self {
        Self {
            serial: SpinLock::new(Serial::new(port)),
        }
    }
}

pub static mut COM1: SyncSerial = SyncSerial::new(0x3F8);

pub fn init() {
    unsafe {
        Serial::init(0x3F8);
    }
}
