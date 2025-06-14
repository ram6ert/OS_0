use bitflags::bitflags;

pub const FRAME_SIZE: usize = 4096;
pub const KERNEL_REGION_BEGIN: usize = 0xffff_8000_0000_0000;
pub const PHYSICAL_MAP_BEGIN: usize = 0xffff_8100_0000_0000;
pub const PHYSICAL_MAP_SIZE: usize = 0x0000_0100_0000_0000;
pub const PHYSICAL_MAP_END: usize = PHYSICAL_MAP_BEGIN + PHYSICAL_MAP_SIZE;
pub const KERNEL_HEAP_BEGIN: usize = 0xffff_8200_0000_0000;
pub const KERNEL_HEAP_SIZE: usize = 16 * 1024 * 1024;
pub const KERNEL_HEAP_END: usize = KERNEL_HEAP_BEGIN + KERNEL_HEAP_SIZE;
pub const KERNEL_STACK_BEGIN: usize = 0xffff_8800_0000_0000;
pub const KERNEL_STACK_SIZE: usize = 0x0000_0100_0000_0000;
pub const KERNEL_ISTACK_END: usize = 0xffff_8900_0000_0000;
pub const APP_STACK_BEGIN: usize = APP_STACK_END - FRAME_SIZE;
pub const APP_STACK_END: usize = 0x8000_0000_0000;

unsafe extern "C" {
    pub static TEXT_START: u64;
    pub static RODATA_START: u64;
    pub static DATA_START: u64;
    pub static BSS_START: u64;
    pub static TEXT_SIZE: u64;
    pub static RODATA_SIZE: u64;
    pub static DATA_SIZE: u64;
    pub static BSS_SIZE: u64;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysAddress(usize);

impl PhysAddress {
    pub fn new(addr: usize) -> Self {
        Self(addr)
    }

    pub fn get_frame(self) -> Frame {
        Frame::new(self.0 / FRAME_SIZE)
    }

    pub fn as_usize(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddress(usize);

impl VirtAddress {
    pub fn new(addr: usize) -> Self {
        Self(addr)
    }

    pub fn get_page(self) -> Page {
        Page::new(self.0 / FRAME_SIZE)
    }

    pub fn as_usize(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame(usize);

impl Into<PhysAddress> for Frame {
    fn into(self) -> PhysAddress {
        PhysAddress::new(self.0 * FRAME_SIZE)
    }
}

impl Frame {
    pub fn new(index: usize) -> Self {
        Self(index)
    }

    pub fn zero() -> Self {
        Self::new(0)
    }

    pub fn is_zero(self) -> bool {
        self.0 == 0
    }

    pub fn get_index(self) -> usize {
        self.0
    }

    pub fn offset(self, o: isize) -> Self {
        Self::new((self.0 as isize + o) as usize)
    }

    pub fn offset_from(self, other: Self) -> isize {
        (self.0 - other.0) as isize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page(usize);

impl Into<VirtAddress> for Page {
    fn into(self) -> VirtAddress {
        VirtAddress(self.0 * FRAME_SIZE)
    }
}

impl Page {
    pub fn new(index: usize) -> Self {
        Self(index)
    }

    pub fn zero() -> Self {
        Self::new(0)
    }

    pub fn is_zero(self) -> bool {
        self.0 == 0
    }

    pub fn get_index(self) -> usize {
        self.0
    }

    pub fn offset(self, o: isize) -> Self {
        Self::new((self.0 as isize + o) as usize)
    }

    pub fn offset_from(self, other: Self) -> isize {
        (self.0 - other.0) as isize
    }
}

#[derive(Debug, Clone)]
pub struct MappingRegion {
    pub phys_begin: Frame,
    pub virt_begin: Page,
    pub num: usize,
}

#[derive(Debug, Clone)]
pub struct PageRegion {
    begin: Page,
    num: usize,
}

impl PageRegion {
    pub fn new(begin: Page, num: usize) -> Self {
        Self { begin, num }
    }

    #[inline]
    pub fn start(&self) -> Page {
        self.begin
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.num
    }

    #[inline]
    pub fn end(&self) -> Page {
        self.begin
    }
}

#[derive(Debug, Clone)]
pub struct FrameRegion {
    begin: Frame,
    num: usize,
}

impl FrameRegion {
    pub fn new(begin: Frame, num: usize) -> Self {
        Self { begin, num }
    }

    #[inline]
    pub fn start(&self) -> Frame {
        self.begin
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.num
    }

    #[inline]
    pub fn end(&self) -> Frame {
        self.begin
    }
}

bitflags! {
    #[derive(Clone, Copy)]
    pub struct PageFlags: u8 {
        const Writable = 1;
        const Usermode = 2;
        const Executable = 4;
    }
}

pub trait PageTable {
    fn map(&mut self, region: &MappingRegion, flags: PageFlags);
    fn unmap(&mut self, region: &PageRegion);
    unsafe fn bind(&self);
    unsafe fn bind_and_switch_stack(&self, sp: usize);
    fn resolve(&self, page: Page) -> Option<Frame>;
}

#[derive(Debug)]
pub enum FrameAllocError {
    OutOfMemory,
    Unknown,
}

#[derive(Debug)]
pub enum FrameFreeError {
    Unknown,
}

pub trait FrameAllocator {
    fn alloc(&mut self, count: usize) -> Result<FrameRegion, FrameAllocError>;
    fn free(&mut self, region: &FrameRegion) -> Result<(), FrameFreeError>;
}
