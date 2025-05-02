use core::{arch::asm, panic::PanicInfo};

use crate::{arch::disable_irq, trace};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    trace!(
        "KERNEL PANICKED!\n{}",
        info.message().as_str().unwrap_or("Unknown reason")
    );
    unsafe {
        disable_irq();
    }
    loop {
        unsafe {
            asm!("hlt");
        }
        core::hint::spin_loop();
    }
}
