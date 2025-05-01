use core::arch::asm;

use crate::mm::{
    definitions::{
        FRAME_SIZE, Frame, FrameAllocator, FrameRegion, MappingRegion, Page, PageFlags, PageRegion,
    },
    frame_allocator::FRAME_ALLOCATOR,
    utils::{borrow_from_phys_addr_mut, calculate_pptr_from_phys_addr},
};

#[derive(Debug)]
pub struct PageTable {
    pml4t: Frame,
}

#[derive(Clone, Copy)]
#[repr(transparent)]
struct TableEntry(u64);

impl TableEntry {
    fn get_present(&self) -> bool {
        self.0 & 1 != 0
    }

    fn set_present(mut self, val: bool) -> Self {
        if val {
            self.0 |= 1u64;
        } else {
            self.0 &= !1u64;
        }
        self
    }

    fn get_writable(&self) -> bool {
        self.0 & (1u64 << 1) != 0
    }

    fn set_writable(mut self, val: bool) -> Self {
        if val {
            self.0 |= 1u64 << 1
        } else {
            self.0 &= !(1u64 << 1);
        }
        self
    }

    fn get_usermode(&self) -> bool {
        self.0 & (1u64 << 2) != 0
    }

    fn set_usermode(mut self, val: bool) -> Self {
        if val {
            self.0 |= 1u64 << 2
        } else {
            self.0 &= !(1u64 << 2);
        }
        self
    }

    fn get_global(&self) -> bool {
        self.0 & (1u64 << 8) != 0
    }

    fn set_global(mut self, val: bool) -> Self {
        if val {
            self.0 |= 1u64 << 8;
        } else {
            self.0 &= !(1u64 << 8);
        }
        self
    }

    fn get_executable(&self) -> bool {
        self.0 & (1u64 << 63) != 0
    }

    fn set_executable(mut self, val: bool) -> Self {
        if val {
            self.0 |= 1u64 << 63;
        } else {
            self.0 &= !(1u64 << 63);
        }
        self
    }

    fn get_frame(&self) -> Frame {
        Frame::new((self.0 >> 12) as usize & ((1usize << 36) - 1))
    }

    fn set_frame(mut self, frame: Frame) -> Self {
        self.0 &= 0xfff0_0000_0000_03ff;
        self.0 |= (frame.get_index() << 12) as u64;
        self
    }

    fn get_huge(self) -> bool {
        self.0 & (1 << 7) != 0
    }

    fn empty() -> Self {
        Self(0)
    }

    fn is_empty(self) -> bool {
        self.0 == 0
    }
}

#[repr(transparent)]
struct TableFrame([TableEntry; 512]);

impl crate::mm::definitions::PageTable for PageTable {
    fn map(&mut self, region: &MappingRegion, flags: PageFlags) {
        let MappingRegion {
            virt_begin,
            phys_begin,
            num,
        } = *region;
        let virt_end = virt_begin.offset(num as isize);
        let mut phys = phys_begin;
        let mut i = virt_begin;

        // in pml4t
        while i != virt_end {
            let pml4t = unsafe { borrow_from_phys_addr_mut::<TableFrame>(self.pml4t.into()) };

            // in pdpt
            let pml4e_idx = Self::get_page_index_4(i);
            let pml4e = &mut pml4t.0[pml4e_idx];
            let pdpt = if pml4e.get_frame().is_zero() {
                let frame = self.alloc_table_frame();
                *pml4e = pml4e.set_frame(frame);
                Self::set_flags(pml4e, flags, false);
                unsafe { borrow_from_phys_addr_mut::<TableFrame>(frame.into()) }
            } else {
                unsafe { borrow_from_phys_addr_mut::<TableFrame>(pml4e.get_frame().into()) }
            };

            while i != virt_end && Self::get_page_index_4(i) == pml4e_idx {
                // in pdt
                let pdpe_idx = Self::get_page_index_3(i);
                let pdpe = &mut pdpt.0[pdpe_idx];
                let pdt = if pdpe.get_frame().is_zero() {
                    let frame = self.alloc_table_frame();
                    *pdpe = pdpe.set_frame(frame);
                    Self::set_flags(pdpe, flags, false);
                    unsafe { borrow_from_phys_addr_mut::<TableFrame>(frame.into()) }
                } else {
                    unsafe { borrow_from_phys_addr_mut::<TableFrame>(pdpe.get_frame().into()) }
                };
                while i != virt_end && Self::get_page_index_3(i) == pdpe_idx {
                    // in pd
                    let pde_idx = Self::get_page_index_2(i);
                    let pde = &mut pdt.0[pde_idx];
                    let pt = if pde.get_frame().is_zero() {
                        let frame = self.alloc_table_frame();
                        *pde = pde.set_frame(frame);
                        Self::set_flags(pde, flags, false);
                        unsafe { borrow_from_phys_addr_mut::<TableFrame>(frame.into()) }
                    } else {
                        unsafe { borrow_from_phys_addr_mut::<TableFrame>(pde.get_frame().into()) }
                    };
                    while i != virt_end && Self::get_page_index_2(i) == pde_idx {
                        let pte_idx = Self::get_page_index_1(i);
                        let pte = &mut pt.0[pte_idx];
                        *pte = pte.set_frame(phys);
                        Self::set_flags(pte, flags, true);

                        i = i.offset(1);
                        phys = phys.offset(1);
                    }
                }
            }
        }
    }

    fn unmap(&mut self, region: &PageRegion) {
        // TODO: unmap unused directory

        let (begin, end) = (region.start(), region.end());
        let mut i = begin;

        let pml4t = unsafe { borrow_from_phys_addr_mut::<TableFrame>(self.pml4t.into()) };
        while i != end {
            let pml4e_idx = Self::get_page_index_4(i);
            let pml4e = &mut pml4t.0[pml4e_idx];
            if pml4e.is_empty() {
                i = i.offset(1);
                continue;
            } else {
                while i != end && Self::get_page_index_4(i) == pml4e_idx {
                    let pdpt = unsafe {
                        borrow_from_phys_addr_mut::<TableFrame>(pml4e.get_frame().into())
                    };
                    let pdpe_idx = Self::get_page_index_3(i);
                    let pdpe = &mut pdpt.0[pdpe_idx];
                    if pdpe.is_empty() {
                        i = i.offset(1);
                        continue;
                    } else {
                        while i != end && Self::get_page_index_3(i) == pdpe_idx {
                            let pdt = unsafe {
                                borrow_from_phys_addr_mut::<TableFrame>(pdpe.get_frame().into())
                            };
                            let pde_idx = Self::get_page_index_2(i);
                            let pde = &mut pdt.0[pde_idx];
                            if pde.is_empty() {
                                i = i.offset(1);
                                continue;
                            } else {
                                while i != end && Self::get_page_index_2(i) == pde_idx {
                                    let pt = unsafe {
                                        borrow_from_phys_addr_mut::<TableFrame>(
                                            pde.get_frame().into(),
                                        )
                                    };
                                    let pt_idx = Self::get_page_index_1(i);
                                    if !pt.0[pt_idx].is_empty() {
                                        pt.0[pt_idx] = TableEntry::empty();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[inline(always)]
    fn bind(&self) {
        unsafe {
            asm!(
                "shl rax, 12",
                "mov cr3, rax",
                in("rax") self.pml4t.get_index())
        };
    }

    fn resolve(&self, page: Page) -> Option<Frame> {
        let pml4e_idx = Self::get_page_index_4(page);
        let pdpe_idx = Self::get_page_index_3(page);
        let pde_idx = Self::get_page_index_2(page);
        let pte_idx = Self::get_page_index_1(page);

        let pml4t = unsafe { borrow_from_phys_addr_mut::<TableFrame>(self.pml4t.into()) };
        let pml4e = pml4t.0[pml4e_idx];
        if !pml4e.get_present() {
            return None;
        }

        let pdpt = unsafe { borrow_from_phys_addr_mut::<TableFrame>(pml4e.get_frame().into()) };
        let pdpe = pdpt.0[pdpe_idx];

        if !pdpe.get_present() {
            return None;
        } else if pdpe.get_huge() {
            return Some(
                pdpe.get_frame()
                    .offset((pde_idx * (1 << 9) + pte_idx) as isize),
            );
        }

        let pdt = unsafe { borrow_from_phys_addr_mut::<TableFrame>(pdpe.get_frame().into()) };
        let pde = pdt.0[pde_idx];
        if !pde.get_present() {
            return None;
        } else if pde.get_huge() {
            return Some(pde.get_frame().offset(pte_idx as isize));
        }

        let pt = unsafe { borrow_from_phys_addr_mut::<TableFrame>(pde.get_frame().into()) };
        let pte = pt.0[pte_idx];
        if !pte.get_present() {
            None
        } else {
            Some(pte.get_frame())
        }
    }
}

impl Drop for PageTable {
    fn drop(&mut self) {
        let (begin, end) = (Page::new(0), Page::new(1usize << 36 - 1));
        let mut i = begin;

        let pml4t = unsafe { borrow_from_phys_addr_mut::<TableFrame>(self.pml4t.into()) };
        while i < end {
            let pml4e_idx = Self::get_page_index_4(i);
            let pml4e = &mut pml4t.0[pml4e_idx];
            if pml4e.get_present() {
                while i < end && Self::get_page_index_4(i) == pml4e_idx {
                    let pdpt = unsafe {
                        borrow_from_phys_addr_mut::<TableFrame>(pml4e.get_frame().into())
                    };
                    let pdpe_idx = Self::get_page_index_3(i);
                    let pdpe = &mut pdpt.0[pdpe_idx];
                    if pdpe.get_present() {
                        if !pdpe.get_huge() {
                            while i < end && Self::get_page_index_3(i) == pdpe_idx {
                                let pdt = unsafe {
                                    borrow_from_phys_addr_mut::<TableFrame>(pdpe.get_frame().into())
                                };
                                let pde_idx = Self::get_page_index_2(i);
                                let pde = &mut pdt.0[pde_idx];
                                // free pt
                                if pde.get_present() && !pde.get_huge() {
                                    FRAME_ALLOCATOR
                                        .lock()
                                        .free(&FrameRegion::new(pde.get_frame(), 1))
                                        .unwrap();
                                }
                                i = Self::index_to_page(&[pml4e_idx, pdpe_idx, pde_idx + 1, 0]);
                            }
                        }

                        // free pdt
                        FRAME_ALLOCATOR
                            .lock()
                            .free(&FrameRegion::new(pdpe.get_frame(), 1))
                            .unwrap();
                    }
                    i = Self::index_to_page(&[pml4e_idx, pdpe_idx + 1, 0, 0]);
                }
                // free pdpt
                FRAME_ALLOCATOR
                    .lock()
                    .free(&FrameRegion::new(pml4e.get_frame(), 1))
                    .unwrap();
            }
            i = Self::index_to_page(&[pml4e_idx + 1, 0, 0, 0]);
        }
        FRAME_ALLOCATOR
            .lock()
            .free(&FrameRegion::new(self.pml4t, 1))
            .unwrap();
    }
}

impl PageTable {
    fn get_page_index_4(page: Page) -> usize {
        page.get_index() >> 27 & 0x1ff
    }

    fn get_page_index_3(page: Page) -> usize {
        page.get_index() >> 18 & 0x1ff
    }

    fn get_page_index_2(page: Page) -> usize {
        page.get_index() >> 9 & 0x1ff
    }

    fn get_page_index_1(page: Page) -> usize {
        page.get_index() & 0x1ff
    }

    fn index_to_page(indices: &[usize; 4]) -> Page {
        let mut idx = indices[0];
        for i in 1..4 {
            idx = (idx << 9) + indices[i];
        }
        Page::new(idx)
    }

    fn alloc_table_frame(&mut self) -> Frame {
        // TODO: drop
        // TODO: no unwrap
        let frame = FRAME_ALLOCATOR.lock().alloc(1).unwrap().start();

        unsafe { borrow_from_phys_addr_mut::<[u8; FRAME_SIZE]>(frame.into()) }.fill(0);
        frame
    }

    fn set_flags(entry: &mut TableEntry, flags: PageFlags, is_leave: bool) {
        *entry = entry
            .set_present(true)
            .set_usermode(flags.contains(PageFlags::Usermode) || !is_leave)
            .set_writable(flags.contains(PageFlags::Writable) || !is_leave)
            .set_executable(/*flags.contains(PageFlags::Executable)*/ false);
    }
}

impl PageTable {
    pub fn new() -> Self {
        let pml4t = FRAME_ALLOCATOR.lock().alloc(1).unwrap().start();
        unsafe {
            core::slice::from_raw_parts_mut(
                calculate_pptr_from_phys_addr::<u8>(pml4t.into()),
                FRAME_SIZE,
            )
            .fill(0);
        }
        Self { pml4t }
    }

    pub fn from(pml4t: Frame) -> Self {
        Self { pml4t }
    }
}
