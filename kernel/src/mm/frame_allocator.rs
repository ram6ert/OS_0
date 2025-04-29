use core::cmp::Ordering;

use lazy_static::lazy_static;

use super::definitions::{Frame, FrameAllocError, FrameFreeError, FrameRegion};

use crate::sync::SpinLock;

pub struct StaticFrameAllocator {
    blocks: [FrameRegion; 64],
}

impl super::definitions::FrameAllocator for StaticFrameAllocator {
    fn alloc(&mut self, count: usize) -> Result<FrameRegion, FrameAllocError> {
        let mut result = None;
        for block in self.blocks.iter_mut() {
            if block.size() >= count {
                let new_block = FrameRegion::new(block.start(), block.size() - count);
                result = Some(FrameRegion::new(
                    block.start().offset(new_block.size() as isize),
                    count,
                ));
                *block = new_block
            } else if block.size() == 0 {
                break;
            }
        }

        self.sort_blocks();

        result.ok_or_else(|| FrameAllocError::OutOfMemory)
    }

    fn free(&mut self, region: &FrameRegion) -> Result<(), FrameFreeError> {
        if self.blocks[63].size() != 0 {
            Err(FrameFreeError::Unknown)
        } else {
            self.blocks[63] = region.clone();
            self.sort_blocks();

            for i in 1..self.blocks.len() {
                if self.blocks[i].size() == 0 {
                    break;
                } else if self.blocks[i - 1].end() == self.blocks[i].start() {
                    self.blocks[i - 1] = FrameRegion::new(
                        self.blocks[i - 1].start(),
                        self.blocks[i - 1].size() + self.blocks[i].size(),
                    );
                    self.blocks[i] = FrameRegion::new(Frame::zero(), 0);
                }
            }

            self.sort_blocks();
            todo!()
        }
    }
}

impl StaticFrameAllocator {
    pub fn new() -> Self {
        Self {
            blocks: core::array::from_fn(|_| FrameRegion::new(Frame::zero(), 0)),
        }
    }

    fn sort_blocks(&mut self) -> () {
        self.blocks.sort_unstable_by(|l, r| {
            if l.size() == 0 && r.size() != 0 {
                Ordering::Greater
            } else {
                l.start().cmp(&r.start())
            }
        });
    }
}

lazy_static! {
    pub static ref FRAME_ALLOCATOR: SpinLock<StaticFrameAllocator> =
        SpinLock::new(StaticFrameAllocator::new());
}
