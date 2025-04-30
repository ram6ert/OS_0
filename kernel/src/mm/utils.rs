use crate::{
    arch::x86_64::mm::page_table::PageTable as X86PageTable,
    mm::{
        definitions::{FRAME_SIZE, FrameAllocator},
        frame_allocator::FRAME_ALLOCATOR,
    },
    sync::SpinLock,
    trace,
};
use core::arch::asm;

use super::definitions::{
    BSS_SIZE, BSS_START, DATA_SIZE, DATA_START, Frame, KERNEL_STACK_BEGIN, MappingRegion,
    PHYSICAL_MAP_BEGIN, PageFlags, PageTable, PhysAddress, RODATA_SIZE, RODATA_START, TEXT_SIZE,
    TEXT_START, VirtAddress,
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

fn get_rsp() -> u64 {
    let rsp: u64;
    unsafe {
        asm!("mov rax, rsp", out("rax") rsp);
    }
    rsp
}

fn create_new_page_table(stack: Frame) -> X86PageTable {
    let old_table_frame = get_current_page_table_frame();
    let old_table = X86PageTable::from(old_table_frame);
    let mut result = X86PageTable::new();

    // 1. kernel region
    let regions = unsafe {
        [
            (TEXT_START, TEXT_SIZE, PageFlags::empty()),
            (RODATA_START, RODATA_SIZE, PageFlags::empty()),
            (DATA_START, DATA_SIZE, PageFlags::Writable),
            (BSS_START, BSS_SIZE, PageFlags::Writable),
        ]
    };
    for region in regions {
        let virt_begin = VirtAddress::new(region.0 as usize).get_page();
        let phys_begin = old_table.resolve(virt_begin).unwrap();
        let num = (region.1 as usize) / FRAME_SIZE;
        result.map(
            &MappingRegion {
                phys_begin,
                virt_begin,
                num,
            },
            region.2,
        );
    }

    // 2. phys region, 128M
    result.map(
        &MappingRegion {
            phys_begin: Frame::zero(),
            virt_begin: VirtAddress::new(PHYSICAL_MAP_BEGIN).get_page(),
            num: 128 * 1024 * 1024 / 4096,
        },
        PageFlags::Writable,
    );

    // 3. int stack, 4K
    let stack_region_begin = VirtAddress::new(KERNEL_STACK_BEGIN + FRAME_SIZE).get_page();
    result.map(
        &MappingRegion {
            phys_begin: stack,
            virt_begin: stack_region_begin,
            num: 1,
        },
        PageFlags::Writable,
    );

    result
}

static FIRST_PAGE_TABLE: SpinLock<Option<X86PageTable>> = SpinLock::new(None);
static INTERRUPTION_STACK: SpinLock<Option<Frame>> = SpinLock::new(None);

pub fn switch_to_new_page_table<F>(callback: F) -> !
where
    F: FnOnce() -> !,
{
    trace!("Creating interruption stack...");
    *INTERRUPTION_STACK.lock() = Some(FRAME_ALLOCATOR.lock().alloc(1).unwrap().start());
    trace!("Success.");
    trace!("Trying to create initial page table...");
    *FIRST_PAGE_TABLE.lock() = Some(create_new_page_table(
        *INTERRUPTION_STACK.lock().as_ref().unwrap(),
    ));
    trace!("Success.");
    trace!("Trying to switch to new page table and switch stack.");
    (*FIRST_PAGE_TABLE.lock()).as_ref().unwrap().bind();
    unsafe {
        asm!(
            "mov rax, {0}",
            "mov rsp, rax",
            const KERNEL_STACK_BEGIN + FRAME_SIZE * 2
        )
    }
    callback()
}
