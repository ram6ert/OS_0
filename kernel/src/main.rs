#![no_main]
#![no_std]
#![feature(allocator_api, slice_ptr_get)]

mod arch;
mod lang_items;
mod mm;
mod sync;

extern crate alloc;

use bootloader_api::BootInfo;
use bootloader_api::entry_point;

entry_point!(kernel_main);

pub fn kernel_main(_boot_info: &'static mut BootInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
