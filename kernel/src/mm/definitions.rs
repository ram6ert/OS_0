use bitflags::bitflags;

pub const FRAME_SIZE: usize = 4096;
pub const PHYSICAL_MAP_BEGIN: usize = 0xffff_8100_0000_0000;
pub const PHYSICAL_MAP_SIZE: usize = 0x0000_0100_0000_0000;
pub const PHYSICAL_MAP_END: usize = PHYSICAL_MAP_BEGIN + PHYSICAL_MAP_SIZE;

#[derive(Debug, Clone, Copy)]
pub struct PhysAddress(usize);

impl Into<usize> for PhysAddress {
    fn into(self) -> usize {
        self.0
    }
}

impl PhysAddress {
    pub fn new(addr: usize) -> Self {
        Self(addr)
    }

    pub fn get_frame(self) -> Frame {
        Frame::new(self.0 / FRAME_SIZE)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VirtAddress(usize);

impl Into<usize> for VirtAddress {
    fn into(self) -> usize {
        self.0
    }
}

impl VirtAddress {
    pub fn new(addr: usize) -> Self {
        Self(addr)
    }

    pub fn get_page(self) -> Page {
        Page::new(self.0 / FRAME_SIZE)
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
    pub struct PageFlags: u8 {
        const Writable = 1;
        const Usermode = 2;
        const Executable = 4;
    }
}

pub trait PageTable {
    fn map(&mut self, region: &MappingRegion, flags: PageFlags);
    fn unmap(&mut self, region: &PageRegion);
    fn bind(&mut self);
}

pub enum FrameAllocError {
    OutOfMemory,
    Unknown,
}

pub enum FrameFreeError {
    Unknown,
}

pub trait FrameAllocator {
    fn alloc(&mut self, count: usize) -> Result<FrameRegion, FrameAllocError>;
    fn free(&mut self, region: &FrameRegion) -> Result<(), FrameFreeError>;
}
