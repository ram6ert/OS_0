use core::arch::asm;

use crate::arch::x86_64::mm::page_table::PageTable as X86PageTable;

use super::definitions::{
    FRAME_SIZE, Frame, KERNEL_REGION_BEGIN, KERNEL_STACK_BEGIN, MappingRegion, PHYSICAL_MAP_BEGIN,
    Page, PageFlags, PageTable, PhysAddress, VirtAddress,
};

pub fn calculate_phys_addr_from_pptr<T>(addr: *mut T) -> PhysAddress {
    let u = addr as usize;
    PhysAddress::new(u - PHYSICAL_MAP_BEGIN)
}

pub fn calculate_pptr_from_phys_addr<T>(addr: PhysAddress) -> *mut T {
    let u = addr.as_usize();
    (u + PHYSICAL_MAP_BEGIN) as *mut T
}

pub unsafe fn borrow_from_phys_addr_mut<T>(addr: PhysAddress) -> &'static mut T {
    unsafe { &mut *calculate_pptr_from_phys_addr::<T>(addr) }
}

fn get_current_page_table_frame() -> Frame {
    let cr3: usize;
    unsafe {
        asm!(
            "mov rax, cr3",
            out("rax") cr3
        );
    }
    Frame::new((cr3 >> 12) & ((1 << 36) - 1))
}

pub fn create_new_page_table() -> X86PageTable {
    let old_table_frame = get_current_page_table_frame();
    let old_table = X86PageTable::from(old_table_frame);
    let mut result = X86PageTable::new();

    // 1. kernel region
    let kernel_region_begin = VirtAddress::new(KERNEL_REGION_BEGIN).get_page();
    let kernel_phys = old_table.resolve(kernel_region_begin).unwrap();
    result.map(
        &MappingRegion {
            phys_begin: kernel_phys,
            virt_begin: kernel_region_begin,
            num: 4 * 1024,
        },
        PageFlags::Writable,
    );

    // 2. phys region, 128M
    result.map(
        &MappingRegion {
            phys_begin: Frame::zero(),
            virt_begin: VirtAddress::new(KERNEL_STACK_BEGIN).get_page(),
            num: 128 * 1024 * 1024 / 4096,
        },
        PageFlags::Writable,
    );

    // 3. kernel stack, 4K
    let stack_region_begin = VirtAddress::new(KERNEL_STACK_BEGIN).get_page();
    let stack_phys = old_table.resolve(stack_region_begin).unwrap();
    result.map(
        &MappingRegion {
            phys_begin: stack_phys,
            virt_begin: stack_region_begin,
            num: 1,
        },
        PageFlags::Writable,
    );

    result
}
