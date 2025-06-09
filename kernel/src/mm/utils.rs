use lazy_static::lazy_static;

use crate::{
    arch::{
        mm::page_table::PageTable as ArchPageTable, x86_64::utils::get_current_page_table_frame,
    },
    mm::{
        definitions::{FRAME_SIZE, FrameAllocator, KERNEL_HEAP_BEGIN, KERNEL_HEAP_SIZE, PageFlags},
        frame_allocator::FRAME_ALLOCATOR,
    },
    sync::SpinLock,
};

use super::definitions::{
    BSS_SIZE, BSS_START, DATA_SIZE, DATA_START, Frame, FrameRegion, MappingRegion,
    PHYSICAL_MAP_BEGIN, PageTable, PhysAddress, RODATA_SIZE, RODATA_START, TEXT_SIZE, TEXT_START,
    VirtAddress,
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

lazy_static! {
    pub static ref INTERRUPTION_STACK: Frame = FRAME_ALLOCATOR.lock().alloc(2).unwrap().start();
}

#[derive(Clone)]
pub struct KernelMappingInfo {
    pub text: MappingRegion,
    pub rodata: MappingRegion,
    pub data: MappingRegion,
    pub bss: MappingRegion,
}

static INITIAL_PAGE_TABLE: SpinLock<Option<ArchPageTable>> = SpinLock::new(None);

pub static KERNEL_MAPPING_INFO: SpinLock<Option<KernelMappingInfo>> = SpinLock::new(None);

lazy_static! {
    pub static ref KERNEL_HEAP: FrameRegion = FRAME_ALLOCATOR
        .lock()
        .alloc(KERNEL_HEAP_SIZE / FRAME_SIZE)
        .unwrap();
}

pub fn init_mm() {
    let mut ipt = INITIAL_PAGE_TABLE.lock();

    *ipt = Some(ArchPageTable::from(unsafe {
        get_current_page_table_frame()
    }));

    let mut kmi = KERNEL_MAPPING_INFO.lock();

    unsafe {
        *kmi = Some(KernelMappingInfo {
            text: MappingRegion {
                phys_begin: ipt
                    .as_ref()
                    .unwrap()
                    .resolve(VirtAddress::new(TEXT_START as usize).get_page())
                    .unwrap(),
                virt_begin: VirtAddress::new(TEXT_START as usize).get_page(),
                num: TEXT_SIZE as usize / FRAME_SIZE,
            },
            rodata: MappingRegion {
                phys_begin: ipt
                    .as_ref()
                    .unwrap()
                    .resolve(VirtAddress::new(RODATA_START as usize).get_page())
                    .unwrap(),
                virt_begin: VirtAddress::new(RODATA_START as usize).get_page(),
                num: RODATA_SIZE as usize / FRAME_SIZE,
            },
            data: MappingRegion {
                phys_begin: ipt
                    .as_ref()
                    .unwrap()
                    .resolve(VirtAddress::new(DATA_START as usize).get_page())
                    .unwrap(),
                virt_begin: VirtAddress::new(DATA_START as usize).get_page(),
                num: DATA_SIZE as usize / FRAME_SIZE,
            },
            bss: MappingRegion {
                phys_begin: ipt
                    .as_ref()
                    .unwrap()
                    .resolve(VirtAddress::new(BSS_START as usize).get_page())
                    .unwrap(),
                virt_begin: VirtAddress::new(BSS_START as usize).get_page(),
                num: BSS_SIZE as usize / FRAME_SIZE,
            },
        });
    }

    ipt.as_mut().unwrap().map(
        &MappingRegion {
            phys_begin: KERNEL_HEAP.start(),
            virt_begin: VirtAddress::new(KERNEL_HEAP_BEGIN).get_page(),
            num: KERNEL_HEAP_SIZE / FRAME_SIZE,
        },
        PageFlags::Writable,
    );

    unsafe {
        ipt.as_ref().unwrap().bind();
    }
}

pub fn free_initial_page_table() {
    *INITIAL_PAGE_TABLE.lock() = None;
}
