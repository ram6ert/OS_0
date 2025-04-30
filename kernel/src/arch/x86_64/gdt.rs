#![allow(dead_code)]

use core::arch::asm;

use crate::trace;

#[repr(transparent)]
struct GdtEntry(u64);

impl GdtEntry {
    const NULL: GdtEntry = GdtEntry(0);
    const KERNEL_CODE: GdtEntry = GdtEntry(0x00AF9A000000FFFF);
    const KERNEL_DATA: GdtEntry = GdtEntry(0x00CF92000000FFFF);
    const USER_CODE: GdtEntry = GdtEntry(0x00AFFA000000FFFF);
}

#[unsafe(link_section = ".ldata")]
static GDT: [GdtEntry; 7] = [
    GdtEntry::NULL,
    GdtEntry::NULL,
    GdtEntry::KERNEL_CODE,
    GdtEntry::NULL,
    GdtEntry::KERNEL_DATA,
    GdtEntry::NULL,
    GdtEntry::USER_CODE,
];

const KERNEL_CODE_DESCRIPTIOR: u16 = 2 * 0x08;
const KERNEL_DATA_DESCRIPTIOR: u16 = 4 * 0x08;
const USER_CODE_DESCRIPTIOR: u16 = 6 * 0x08;

#[repr(C, packed)]
struct Gdtr {
    size: u16,
    ptr: u64,
}

pub unsafe fn load_gdt() {
    let size = (size_of_val(&GDT) - 1) as u16;
    let ptr = GDT.as_ptr() as u64;
    let gdtr = Gdtr { size, ptr };
    unsafe {
        asm!(
            "lgdt [{0}]",
            "mov ax, {2}",
            "mov ss, ax",
            "push {1}",
            "lea {0}, [rip + 2f]",
            "push {0}",
            "retfq",
            "2:",
            "mov ax, {2}",
            "mov ds, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov es, ax",
            in(reg) &gdtr,
            const KERNEL_CODE_DESCRIPTIOR,
            const KERNEL_DATA_DESCRIPTIOR,
        );
    }
}
