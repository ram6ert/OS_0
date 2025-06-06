use core::arch::naked_asm;

use crate::{
    arch::{
        RegisterStore as ArchRegisterStore, enable_external_irq, enable_irq,
        mm::page_table::PageTable as ArchPageTable,
        x86_64::task::{jump_to_user, set_structure_base},
    },
    mm::{
        INTERRUPTION_STACK,
        definitions::{
            FRAME_SIZE, Frame, FrameAllocator, KERNEL_HEAP_BEGIN, KERNEL_HEAP_SIZE,
            KERNEL_ISTACK_END, KERNEL_STACK_BEGIN, MappingRegion, PHYSICAL_MAP_BEGIN, PageFlags,
            PageTable, VirtAddress,
        },
        frame_allocator::FRAME_ALLOCATOR,
        utils::{KERNEL_HEAP, KERNEL_MAPPING_INFO},
    },
};

pub trait RegisterStore {
    extern "sysv64" fn switch_to(&self) -> !;
    fn new(pc: usize, sp: usize, ksp: usize) -> Self;
}

#[repr(C)]
pub struct Task {
    pub registers: ArchRegisterStore,
    page_table: ArchPageTable,
    id: usize,
}

impl Task {
    pub fn new(id: usize, entry: usize) -> Self {
        Self {
            registers: ArchRegisterStore::new(
                entry,
                0,
                KERNEL_STACK_BEGIN + (2 * id + 1) * FRAME_SIZE,
            ),
            page_table: Self::create_page_table(id),
            id,
        }
    }

    fn create_page_table(id: usize) -> ArchPageTable {
        let mut result = ArchPageTable::new();

        // 1. kernel regions
        let kmi = KERNEL_MAPPING_INFO.lock().as_ref().unwrap().clone();
        let regions = [
            (kmi.text, PageFlags::Executable | PageFlags::Usermode),
            (kmi.rodata, PageFlags::empty()),
            (kmi.data, PageFlags::Writable),
            (kmi.bss, PageFlags::Writable),
        ];

        for region in regions {
            result.map(&region.0, region.1);
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
        let istack_region_begin = VirtAddress::new(KERNEL_ISTACK_END - 1).get_page();
        let istack = *INTERRUPTION_STACK;
        result.map(
            &MappingRegion {
                phys_begin: istack,
                virt_begin: istack_region_begin,
                num: 1,
            },
            PageFlags::Writable,
        );

        // 4. task kernel stack, 4K
        let stack_region_begin = VirtAddress::new(KERNEL_STACK_BEGIN)
            .get_page()
            .offset(2 * id as isize + 1);
        let stack = FRAME_ALLOCATOR.lock().alloc(1).unwrap().start();
        result.map(
            &MappingRegion {
                phys_begin: stack,
                virt_begin: stack_region_begin,
                num: 1,
            },
            PageFlags::Writable,
        );

        // 5. kernel heap, 16M
        let kernel_heap_region_begin = VirtAddress::new(KERNEL_HEAP_BEGIN).get_page();
        result.map(
            &MappingRegion {
                phys_begin: KERNEL_HEAP.start(),
                virt_begin: kernel_heap_region_begin,
                num: KERNEL_HEAP_SIZE / FRAME_SIZE,
            },
            PageFlags::Writable,
        );
        result
    }

    #[inline(always)]
    pub unsafe fn jump_to(&self) -> ! {
        unsafe {
            set_structure_base(self as *const Task as u64, true);
            self.page_table.bind();
            self.registers.switch_to();
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }
}

#[naked]
unsafe extern "C" fn idle() -> ! {
    unsafe {
        naked_asm!("syscall", "2:", "jmp 2b");
    }
}
