pub const FRAME_SIZE: usize = 4096;

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

#[derive(Debug, Clone, Copy)]
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
}

#[derive(Debug, Clone, Copy)]
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
}
