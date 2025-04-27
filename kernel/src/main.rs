#![no_main]
#![no_std]

mod lang_items;
mod mm;

use bootloader_api::BootInfo;
use bootloader_api::entry_point;

entry_point!(kernel_main);

pub fn kernel_main(_boot_info: &'static mut BootInfo) -> ! {
    loop {
        //core::hint::spin_loop();
        core::hint::spin_loop();
    }
}
