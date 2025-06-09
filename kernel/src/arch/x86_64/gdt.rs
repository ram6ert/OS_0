#![allow(dead_code)]

use core::{arch::asm, u16};

use crate::{mm::definitions::KERNEL_ISTACK_END, trace};

#[repr(transparent)]
struct GdtEntry(u64);

impl GdtEntry {
    const NULL: GdtEntry = GdtEntry(0);
    const KERNEL_CODE: GdtEntry = GdtEntry(0x00AF9A000000FFFF);
    const KERNEL_DATA: GdtEntry = GdtEntry(0x00CF92000000FFFF);
    const USER_CODE: GdtEntry = GdtEntry(0x00AFFA000000FFFF);
    const USER_DATA: GdtEntry = GdtEntry(0x00CFF2000000FFFF);
}

#[repr(C, packed)]
struct TssEntry {
    _reserved_0: u32,
    rsp0: u64,
    rsp1: u64,
    rsp2: u64,
    _reserved_1: u64,
    ists: [u64; 7],
    _reverved_2: u64,
    _reserved_3: u16,
    io_map_base: u16,
}

impl TssEntry {
    const fn new(kernel_stack: u64) -> Self {
        Self {
            _reserved_0: 0,
            rsp0: kernel_stack,
            rsp1: 0,
            rsp2: 0,
            _reserved_1: 0,
            ists: [0; 7],
            _reverved_2: 0,
            _reserved_3: 0,
            io_map_base: u16::MAX,
        }
    }
}

static mut GDT: [GdtEntry; 8] = [
    GdtEntry::NULL,
    GdtEntry::KERNEL_CODE,
    GdtEntry::KERNEL_DATA,
    GdtEntry::NULL,
    GdtEntry::USER_DATA,
    GdtEntry::USER_CODE,
    // for tss
    GdtEntry::NULL,
    GdtEntry::NULL,
];

#[unsafe(link_section = ".ldata")]
static TSS: TssEntry = TssEntry::new(KERNEL_ISTACK_END as u64);

pub const KERNEL_CODE_DESCRIPTOR: u16 = 1 * 0x08;
pub const KERNEL_DATA_DESCRIPTOR: u16 = 2 * 0x08;
pub const BEFORE_USER_DESCRIPTOR: u16 = 3 * 0x08;
pub const USER_CODE_DESCRIPTOR: u16 = 5 * 0x08;
pub const USER_DATA_DESCRIPTOR: u16 = 4 * 0x08;
pub const TSS_DESCRIPTOR: u16 = 6 * 0x08;

#[repr(C, packed)]
struct Gdtr {
    size: u16,
    ptr: u64,
}

pub unsafe fn load_gdt() {
    trace!("Loading GDT...");
    let tss_base = &TSS as *const TssEntry as u64;
    let tss_limit = (size_of_val(&TSS) - 1) as u64;
    unsafe {
        GDT[6] =
            GdtEntry((tss_limit & 0xffff) | ((tss_base & 0xffffff) << 16) | 0x0000890000000000u64);
        GDT[7] = GdtEntry((tss_base >> 32) as u64);
    }

    #[allow(static_mut_refs)]
    let (size, ptr) = unsafe {
        let size = (size_of_val(&GDT) - 1) as u16;
        let ptr = GDT.as_ptr() as u64;
        (size, ptr)
    };
    let gdtr = Gdtr { size, ptr };
    unsafe {
        asm!(
            "lgdt [{0}]",
            "mov ax, {2}",
            "mov ss, ax",
            "push {1}",
            "lea rax, [rip + 2f]",
            "push rax",
            "retfq",
            "2:",
            "mov ax, {2}",
            "mov ds, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov es, ax",
            "mov ax, {3}",
            "ltr ax",
            in(reg) &gdtr,
            const KERNEL_CODE_DESCRIPTOR,
            const KERNEL_DATA_DESCRIPTOR,
            const TSS_DESCRIPTOR,
            out("rax") _,
        );
    }
    trace!("GDT loaded.");
}
