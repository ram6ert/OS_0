#![no_main]
#![no_std]
#![feature(allocator_api, slice_ptr_get)]

mod arch;
mod lang_items;
mod mm;
mod sync;

extern crate alloc;

use bootloader_api::BootInfo;
use bootloader_api::BootloaderConfig;
use bootloader_api::config::Mapping;
use bootloader_api::entry_point;
use mm::definitions::KERNEL_STACK_BEGIN;
use mm::definitions::PHYSICAL_MAP_BEGIN;

static CONFIG: BootloaderConfig = {
    let mut cfg = BootloaderConfig::new_default();
    cfg.mappings.physical_memory = Some(Mapping::FixedAddress(PHYSICAL_MAP_BEGIN as u64));
    cfg.mappings.kernel_stack = Mapping::FixedAddress(KERNEL_STACK_BEGIN as u64);
    cfg
};

entry_point!(kernel_main, config = &CONFIG);

pub fn kernel_main(_boot_info: &'static mut BootInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
