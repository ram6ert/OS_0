#![no_main]
#![no_std]
#![feature(
    allocator_api,
    slice_ptr_get,
    naked_functions,
    never_type,
    linked_list_cursors
)]

mod arch;
mod lang_items;
mod mm;
mod sync;
mod task;
mod user;

extern crate alloc;

use bootloader_api::BootInfo;
use bootloader_api::BootloaderConfig;
use bootloader_api::config::Mapping;
use bootloader_api::entry_point;
use bootloader_api::info::MemoryRegionKind;
use mm::definitions::FRAME_SIZE;
use mm::definitions::FrameAllocator;
use mm::definitions::FrameRegion;
use mm::definitions::PHYSICAL_MAP_BEGIN;
use mm::definitions::PhysAddress;
use mm::frame_allocator::FRAME_ALLOCATOR;
use mm::utils::init_mm;

use crate::task::init_first_process_and_jump_to;

static CONFIG: BootloaderConfig = {
    let mut cfg = BootloaderConfig::new_default();
    cfg.mappings.physical_memory = Some(Mapping::FixedAddress(PHYSICAL_MAP_BEGIN as u64));
    cfg
};

entry_point!(kernel_boot, config = &CONFIG);

pub static INIT_PROGRAM: &[u8] = include_bytes!("../artifacts/user");

pub fn kernel_boot(boot_info: &'static mut BootInfo) -> ! {
    arch::init();
    for region in boot_info
        .memory_regions
        .iter()
        .filter(|x| x.kind == MemoryRegionKind::Usable)
    {
        let frame_begin = PhysAddress::new(region.start as usize + FRAME_SIZE - 1).get_frame();
        let frame_end = PhysAddress::new(region.end as usize - FRAME_SIZE + 1).get_frame();
        FRAME_ALLOCATOR
            .lock()
            .free(&FrameRegion::new(
                frame_begin,
                frame_end.offset_from(frame_begin) as usize - 1,
            ))
            .unwrap();
    }
    init_mm();
    init_first_process_and_jump_to()
}
