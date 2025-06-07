#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};

fn syscall() {
    unsafe { asm!("syscall") }
}

fn delay() {
    unsafe { asm!("mov rcx, 0xffffffff", "634:", "loop 634b", out("rcx") _) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn _start() -> ! {
    loop {
        syscall();
        delay();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
