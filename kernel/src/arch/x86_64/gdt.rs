#![allow(dead_code)]

use core::arch::asm;

#[repr(transparent)]
struct GdtEntry(u64);

impl GdtEntry {
    const NULL: GdtEntry = GdtEntry(0);
    const KERNEL_CODE: GdtEntry = GdtEntry(0x00AF9A000000FFFF);
    const KERNEL_DATA: GdtEntry = GdtEntry(0x00CF92000000FFFF);
    const USER_CODE: GdtEntry = GdtEntry(0x00AFFA000000FFFF);
    const USER_DATA: GdtEntry = GdtEntry(0x00CFF2000000FFFF);
}

#[unsafe(link_section = ".ldata")]
static GDT: [GdtEntry; 5] = [
    GdtEntry::NULL,
    GdtEntry::KERNEL_CODE,
    GdtEntry::KERNEL_DATA,
    GdtEntry::USER_CODE,
    GdtEntry::USER_DATA,
];

pub const KERNEL_CODE_DESCRIPTOR: u16 = 1 * 0x08;
pub const KERNEL_DATA_DESCRIPTOR: u16 = 2 * 0x08;
pub const USER_CODE_DESCRIPTOR: u16 = 3 * 0x08;
pub const USER_DATA_DESCRIPTOR: u16 = 4 * 0x08;

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
            "lea rax, [rip + 2f]",
            "push rax",
            "retfq",
            "2:",
            "mov ax, {2}",
            "mov ds, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov es, ax",
            in(reg) &gdtr,
            const KERNEL_CODE_DESCRIPTOR,
            const KERNEL_DATA_DESCRIPTOR,
            out("rax") _,
        );
    }
}
