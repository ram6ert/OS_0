use super::serial::{self, COM1, Serial};

#[macro_export]
macro_rules! trace {
    ($($arg: tt)*) => {
        {
            use core::fmt::Write;
            unsafe {
                #[allow(static_mut_refs)]
                core::writeln!(crate::arch::x86_64::serial::COM1, $($arg)*).unwrap();
            }
        }
    };
}

pub fn init() {
    serial::init();
}
