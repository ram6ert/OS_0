use core::{arch::asm, panic::PanicInfo};

use crate::trace;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    trace!(
        "KERNEL PANICKED!\n{}",
        info.message().as_str().unwrap_or("Unknown reason")
    );
    loop {
        unsafe {
            asm!("hlt");
        }
        core::hint::spin_loop();
    }
}
