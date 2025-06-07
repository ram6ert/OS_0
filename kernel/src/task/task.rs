use crate::{
    arch::{
        RegisterStore as ArchRegisterStore, mm::page_table::PageTable as ArchPageTable,
        x86_64::task::set_structure_base,
    },
    mm::{
        INTERRUPTION_STACK,
        definitions::{
            APP_STACK_BEGIN, APP_STACK_END, FRAME_SIZE, Frame, FrameAllocator, KERNEL_HEAP_BEGIN,
            KERNEL_HEAP_SIZE, KERNEL_ISTACK_END, KERNEL_STACK_BEGIN, MappingRegion,
            PHYSICAL_MAP_BEGIN, PageFlags, PageTable, VirtAddress,
        },
        frame_allocator::FRAME_ALLOCATOR,
        utils::{KERNEL_HEAP, KERNEL_MAPPING_INFO},
    },
    task::elf::{Readable, load_elf},
};

pub trait RegisterStore {
    extern "sysv64" fn switch_to(&self) -> !;
    fn ksp(&self) -> usize;
    fn new(pc: usize, sp: usize, ksp: usize) -> Self;

    fn update(&mut self, pc: usize, sp: usize);
}

#[repr(C)]
pub struct Task {
    pub registers: ArchRegisterStore,
    page_table: ArchPageTable,
    id: usize,
}

impl Task {
    pub fn new<R: Readable>(id: usize, elf_file: R) -> Self {
        let mut page_table = Self::create_page_table(id);
        let entry = load_elf(elf_file, &mut page_table);
        let registers = ArchRegisterStore::new(
            entry as usize,
            APP_STACK_END,
            KERNEL_STACK_BEGIN + (2 * id + 2) * FRAME_SIZE,
        );
        Self {
            registers,
            page_table,
            id,
        }
    }

    fn create_page_table(id: usize) -> ArchPageTable {
        let mut result = ArchPageTable::new();

        // 1. kernel regions
        let kmi = KERNEL_MAPPING_INFO.lock().as_ref().unwrap().clone();
        let regions = [
            (kmi.text, PageFlags::Executable),
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
        let kstack = FRAME_ALLOCATOR.lock().alloc(1).unwrap().start();
        result.map(
            &MappingRegion {
                phys_begin: kstack,
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

        // 6. app stack
        let stack = FRAME_ALLOCATOR.lock().alloc(1).unwrap().start();
        result.map(
            &MappingRegion {
                phys_begin: stack,
                virt_begin: VirtAddress::new(APP_STACK_BEGIN).get_page(),
                num: 1,
            },
            PageFlags::Usermode | PageFlags::Writable,
        );
        result
    }

    #[inline(always)]
    pub unsafe fn jump_to(&self) -> ! {
        unsafe {
            set_structure_base(self as *const Task as u64, false);
            self.page_table.bind_and_switch_stack(self.registers.ksp());
            self.registers.switch_to();
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }
}
