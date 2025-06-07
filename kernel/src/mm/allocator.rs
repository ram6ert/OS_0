use core::alloc::{GlobalAlloc, Layout};
use core::{cmp::Ordering, usize};

use crate::sync::SpinLock;

use super::definitions::{KERNEL_HEAP_BEGIN, KERNEL_HEAP_SIZE};

#[derive(Debug, Clone)]
struct MemoryHole {
    base: usize,
    size: usize,
}

struct DirectBlockAllocator {
    base: *mut u8,
    floor: usize,
    ceiling: usize,
}

impl DirectBlockAllocator {
    const fn new(base: *mut u8, size: usize) -> Self {
        Self {
            base,
            floor: 0,
            ceiling: size,
        }
    }

    fn alloc(&mut self, layout: Layout) -> *mut u8 {
        unsafe {
            self.try_alloc_from_ceiling(layout)
                .or(self.try_alloc_from_hole(layout))
                .map(|x| self.base.offset(x as isize))
                .unwrap_or(core::ptr::null_mut())
        }
    }

    fn free(&mut self, ptr: *mut u8, layout: Layout) {
        Self::insert_hole(
            self,
            &MemoryHole {
                base: unsafe { ptr.offset_from(self.base) as usize },
                size: layout.size(),
            },
        )
        .unwrap();
    }

    fn try_alloc_from_ceiling(&mut self, layout: Layout) -> Option<usize> {
        let size = layout.size();
        let align = layout.align();

        let mut result = None;
        let rest = self.ceiling - self.floor;
        if rest >= size {
            let mut cand = self.ceiling - size;
            if cand % align != 0 {
                let offset = cand % align;

                if rest >= size + offset + size_of::<MemoryHole>() {
                    cand = cand - offset;
                    let hole = MemoryHole {
                        base: self.ceiling - offset,
                        size: offset,
                    };
                    Self::insert_hole(self, &hole).unwrap();
                    result = Some(cand);
                    self.ceiling -= size + offset;
                }
            } else {
                result = Some(cand);
                self.ceiling -= size;
            }
        }

        result
    }

    fn try_alloc_from_hole(&mut self, layout: Layout) -> Option<usize> {
        let hole_count = self.floor / size_of::<MemoryHole>();
        let holes =
            unsafe { core::slice::from_raw_parts_mut(self.base as *mut MemoryHole, hole_count) };

        let size = layout.size();
        let align = layout.align();

        let mut result = None;
        let mut to_delete = usize::MAX;

        for (i, hole) in holes.iter_mut().enumerate() {
            if hole.size >= size {
                let mut before = None;
                let mut after = None;
                let cand = hole.base;
                if cand % align != 0 {
                    let offset = align - cand % align;
                    if hole.size >= size + offset {
                        result = Some(cand + offset);

                        if size + offset == hole.size {
                            before = Some(MemoryHole {
                                base: hole.base,
                                size: offset,
                            });
                        }
                    }
                } else {
                    result = Some(cand)
                }

                if result.is_some() {
                    if result.unwrap() + size < hole.base + hole.size {
                        after = Some(MemoryHole {
                            base: result.unwrap() + size,
                            size: hole.base + hole.size - result.unwrap() - size,
                        })
                    }

                    if before.is_some() && after.is_some() {
                        *hole = before.unwrap();
                        Self::insert_hole(self, &after.unwrap()).unwrap();
                    } else {
                        let new_hole = before.or(after);
                        if let Some(nh) = new_hole {
                            *hole = nh;
                        } else {
                            to_delete = i;
                        }
                    }
                    break;
                }
            }
        }

        if to_delete != usize::MAX {
            holes[to_delete..].rotate_left(1);
            self.floor -= size_of::<MemoryHole>()
        }
        result
    }

    fn insert_hole(&mut self, hole: &MemoryHole) -> Result<(), ()> {
        if self.ceiling - self.floor < size_of::<MemoryHole>() {
            Err(())
        } else {
            let hole_count = self.floor / size_of::<MemoryHole>();
            let holes = unsafe {
                core::slice::from_raw_parts_mut(self.base as *mut MemoryHole, hole_count + 1)
            };
            holes[hole_count] = hole.clone();
            holes.sort_unstable_by(|l, r| l.base.cmp(&r.base).reverse());
            self.floor += size_of::<MemoryHole>();
            Self::merge_holes(self);
            Ok(())
        }
    }

    fn merge_holes(&mut self) {
        let hole_count = self.floor / size_of::<MemoryHole>();
        let holes =
            unsafe { core::slice::from_raw_parts_mut(self.base as *mut MemoryHole, hole_count) };

        let mut num = 0;
        for i in 0..hole_count {
            if holes[i].base == self.ceiling {
                self.ceiling -= holes[i].size;
                holes[i].size = 0;
                num += 1;
            } else if i < hole_count - 1 && holes[i].base + holes[i].size == holes[i + 1].base {
                holes[i + 1].base = holes[i].base;
                holes[i + 1].size += holes[i].size;
                holes[i].size = 0;
                num += 1;
            }
        }

        holes.sort_unstable_by(|l, r| {
            if l.size == 0 && r.size != 0 {
                Ordering::Greater
            } else if l.size != 0 && r.size == 0 {
                Ordering::Less
            } else {
                l.base.cmp(&r.base).reverse()
            }
        });

        self.floor -= num * size_of::<MemoryHole>();
    }
}

struct GlobalAllocator {
    allocator: SpinLock<DirectBlockAllocator>,
}

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.allocator.lock().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.allocator.lock().free(ptr, layout);
    }
}

impl GlobalAllocator {
    const fn new() -> Self {
        Self {
            allocator: SpinLock::new(DirectBlockAllocator::new(
                KERNEL_HEAP_BEGIN as *mut u8,
                KERNEL_HEAP_SIZE,
            )),
        }
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator::new();
