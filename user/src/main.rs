#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};

fn write(fp: usize, data: &[u8]) -> usize {
    let mut result: usize = 1;
    unsafe {
        asm!(
            "syscall",
            inout("rax") result,
            in("rdi") fp,
            in("rsi") data.as_ptr(),
            in("rdx") data.len(),
            out("rcx") _,
            out("r11") _,
        )
    }
    result
}

fn getpid() -> usize {
    let mut result: usize = 3;
    unsafe {
        asm!("syscall", inout("rax") result, out("rcx") _, out("r11") _);
    }
    result
}

fn spawn() -> usize {
    let mut result: usize = 4;
    unsafe {
        asm!("syscall", inout("rax") result, out("rcx") _, out("r11") _);
    }
    result
}

fn delay() {
    unsafe { asm!("mov rcx, 0xffffff", "634:", "loop 634b", out("rcx") _) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn _start() -> ! {
    let pid = getpid();
    if pid == 1 {
        spawn();
    }
    loop {
        write(1, &[(pid + 48) as u8, '\n' as u8]);
        delay();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
