use bitflags::bitflags;

use crate::{
    arch::mm::page_table::PageTable as ArchPageTable,
    mm::{
        definitions::{
            FRAME_SIZE, FrameAllocator, MappingRegion, PageFlags, PageTable, VirtAddress,
        },
        frame_allocator::FRAME_ALLOCATOR,
        utils::borrow_from_phys_addr_mut,
    },
};

#[repr(C)]
#[derive(Debug)]
struct ElfHeader {
    ident: [u8; 16],
    tybe: u16,
    machine: u16,
    version: u32,
    entry: u64,
    phoff: u64,
    shoff: u64,
    flags: u32,
    ehsize: u16,
    phentsize: u16,
    phnum: u16,
    shentsize: u16,
    shnum: u16,
    shstrndx: u16,
}

bitflags! {
    #[derive(Debug)]
    struct ProgramFlags: u32 {
        const Executable = 1;
        const Readable = 2;
        const Writable = 4;
    }
}

#[repr(C)]
#[derive(Debug)]
struct ProgramHeader {
    tybe: u32,
    flags: ProgramFlags,
    offset: u64,
    vaddr: u64,
    paddr: u64,
    filesz: u64,
    memsz: u64,
    align: u64,
}

pub trait Readable {
    fn read(&self, dst: *mut u8, offset: usize, size: usize) -> Result<(), ReadError>;
}

#[derive(Debug)]
pub enum ReadError {
    OutOfBound,
}

pub struct MemoryReader {
    ptr: *const u8,
    size: usize,
}

pub fn load_elf<R: Readable>(reader: R, pt: &mut ArchPageTable) -> u64 {
    let mut header: ElfHeader = unsafe { core::mem::zeroed() };
    reader
        .read(
            &mut header as *mut ElfHeader as *mut u8,
            0,
            size_of::<ElfHeader>(),
        )
        .expect("Failure reading elf header.");

    for ph_idx in 0..header.phnum {
        let ph_offset = header.phoff as u64 + ph_idx as u64 * header.phentsize as u64;
        let mut ph: ProgramHeader = unsafe { core::mem::zeroed() };
        reader
            .read(
                &mut ph as *mut ProgramHeader as *mut u8,
                ph_offset as usize,
                size_of::<ProgramHeader>(),
            )
            .expect("Failure reading program header.");

        if ph.tybe != 1 {
            continue;
        }

        let frames = FRAME_ALLOCATOR
            .lock()
            .alloc((ph.memsz as usize + FRAME_SIZE - 1) / FRAME_SIZE)
            .expect("Cannot allocate frames for program.");

        let mut flags = PageFlags::Usermode;
        if ph.flags.contains(ProgramFlags::Executable) {
            flags.toggle(PageFlags::Executable);
        }
        if ph.flags.contains(ProgramFlags::Writable) {
            flags.toggle(PageFlags::Writable);
        }
        if ph.flags.contains(ProgramFlags::Readable) {
            // nothing
        }

        pt.map(
            &MappingRegion {
                phys_begin: frames.start(),
                virt_begin: VirtAddress::new(ph.vaddr as usize).get_page(),
                num: frames.size(),
            },
            flags,
        );

        unsafe {
            reader
                .read(
                    (borrow_from_phys_addr_mut(frames.start().into()) as *mut u8)
                        .offset(ph.vaddr as isize % FRAME_SIZE as isize),
                    ph.offset as usize,
                    ph.filesz as usize,
                )
                .expect("Failure reading program segments.");
            core::ptr::write_bytes(
                (borrow_from_phys_addr_mut(frames.start().into()) as *mut u8)
                    .offset(ph.filesz as isize + ph.vaddr as isize % FRAME_SIZE as isize),
                0,
                (ph.memsz - ph.filesz) as usize,
            );
        }
    }

    header.entry
}

impl MemoryReader {
    pub fn new(ptr: *const u8, size: usize) -> Self {
        Self { ptr, size }
    }
}

impl Readable for MemoryReader {
    fn read(&self, dst: *mut u8, offset: usize, size: usize) -> Result<(), ReadError> {
        if offset + size >= self.size {
            Err(ReadError::OutOfBound)
        } else {
            unsafe {
                core::ptr::copy(self.ptr.offset(offset as isize), dst, size);
            }
            Ok(())
        }
    }
}
