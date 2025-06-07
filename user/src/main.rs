#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};

fn syscall() {
    unsafe { asm!("syscall") }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn _start() -> ! {
    syscall();
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
