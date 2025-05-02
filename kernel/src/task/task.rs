use core::arch::naked_asm;

use crate::{
    arch::{
        RegisterStore, disable_irq, enable_external_irq, enable_irq,
        mm::page_table::PageTable as ArchPageTable,
        x86_64::{
            syscall,
            task::{jump_to, set_structure_base},
            utils::bind_pt_and_switch_stack,
        },
    },
    mm::{
        INTERRUPTION_STACK,
        definitions::{
            FRAME_SIZE, Frame, FrameAllocator, KERNEL_ISTACK_END, KERNEL_STACK_BEGIN,
            MappingRegion, PHYSICAL_MAP_BEGIN, PageFlags, PageTable, VirtAddress,
        },
        frame_allocator::FRAME_ALLOCATOR,
        utils::{KERNEL_MAPPING_INFO, free_initial_pt},
    },
    sync::SpinLock,
    trace,
};
#[repr(C)]
pub struct Task {
    registers: RegisterStore,
    pc: usize,
    page_table: ArchPageTable,
}

impl Task {
    pub fn new(stack_idx: usize, entry: usize) -> Self {
        Self {
            registers: RegisterStore::new(
                (KERNEL_STACK_BEGIN + (2 * stack_idx + 1) * FRAME_SIZE) as u64,
            ),
            page_table: Self::create_page_table(stack_idx),
            pc: entry,
        }
    }

    fn from_pt(stack_idx: usize, entry: usize, pt: ArchPageTable) -> Self {
        Self {
            registers: RegisterStore::new(
                (KERNEL_STACK_BEGIN + (2 * stack_idx + 1) * FRAME_SIZE) as u64,
            ),
            page_table: pt,
            pc: entry,
        }
    }

    fn create_page_table(stack_idx: usize) -> ArchPageTable {
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

        // 4. current stack, 4K
        let stack_region_begin = VirtAddress::new(KERNEL_STACK_BEGIN)
            .get_page()
            .offset(2 * stack_idx as isize + 1);
        let stack = FRAME_ALLOCATOR.lock().alloc(1).unwrap().start();
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

    pub unsafe fn jump_to(&self) -> ! {
        unsafe {
            set_structure_base(self as *const Task as u64);
            self.page_table.bind();
            jump_to(self.pc as u64);
        }
    }
}

#[naked]
unsafe extern "C" fn idle() -> ! {
    unsafe {
        naked_asm!("syscall", "2:", "jmp 2b");
    }
}

static IDLE: SpinLock<Option<Task>> = SpinLock::new(None);
static IDLE_PT: SpinLock<Option<ArchPageTable>> = SpinLock::new(None);

pub fn jump_idle() -> ! {
    unsafe {
        *IDLE_PT.get_mut() = Some(Task::create_page_table(0));
        bind_pt_and_switch_stack(
            IDLE_PT.get_mut().as_ref().unwrap(),
            (KERNEL_STACK_BEGIN + FRAME_SIZE * 2) as u64,
            || {
                enable_irq();
                enable_external_irq();
                *IDLE.get_mut() = Some(Task::from_pt(
                    0,
                    idle as usize,
                    IDLE_PT.get_mut().take().unwrap(),
                ));
                IDLE.get_mut().as_ref().unwrap().jump_to();
            },
        );
    }
}
