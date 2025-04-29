#![no_main]
#![no_std]
#![feature(allocator_api, slice_ptr_get)]

mod arch;
mod lang_items;
mod mm;
mod sync;

extern crate alloc;

use arch::x86_64::logging;
use bootloader_api::BootInfo;
use bootloader_api::BootloaderConfig;
use bootloader_api::config::Mapping;
use bootloader_api::entry_point;
use bootloader_api::info::MemoryRegionKind;
use mm::definitions::FRAME_SIZE;
use mm::definitions::FrameAllocator;
use mm::definitions::FrameRegion;
use mm::definitions::KERNEL_STACK_BEGIN;
use mm::definitions::PHYSICAL_MAP_BEGIN;
use mm::definitions::PageTable;
use mm::definitions::PhysAddress;
use mm::frame_allocator::FRAME_ALLOCATOR;
use mm::utils::create_new_page_table;

static CONFIG: BootloaderConfig = {
    let mut cfg = BootloaderConfig::new_default();
    cfg.mappings.physical_memory = Some(Mapping::FixedAddress(PHYSICAL_MAP_BEGIN as u64));
    cfg.mappings.kernel_stack = Mapping::FixedAddress(KERNEL_STACK_BEGIN as u64);
    cfg
};

entry_point!(kernel_main, config = &CONFIG);

pub fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    logging::init();
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

    trace!("Begin to create new page table.");
    let mut pt = create_new_page_table();
    //pt.bind();
    loop {
        core::hint::spin_loop();
    }
}
