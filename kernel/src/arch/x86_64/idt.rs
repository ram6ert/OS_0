#![allow(dead_code)]

use super::gdt;
use core::arch::{asm, naked_asm};
use lazy_static::lazy_static;

use crate::{arch::x86_64::int::send_eoi, trace};

#[repr(u8)]
enum GateType {
    InterruptGate = 0b1110,
    TrapGate = 0b1111,
}

#[repr(u8)]
enum PrivilegeLevel {
    Ring0 = 0,
    Ring1 = 1,
    Ring2 = 2,
    Ring3 = 3,
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct IdtEntry {
    offset_low: u16,
    segment_selector: u16,
    options_0: u8,
    options_1: u8,
    offset_middle: u16,
    offset_high: u32,
    reserved: u32,
}

impl IdtEntry {
    const fn new(
        offset: u64,
        segment_selector: u16,
        ty: GateType,
        dpl: PrivilegeLevel,
        present: bool,
    ) -> Self {
        let _present = if present { 1 } else { 0 };
        Self {
            offset_low: (offset & 0x000000000000ffff) as u16,
            segment_selector,
            options_0: 0,
            options_1: (ty as u8) | ((dpl as u8) << 5) | (_present << 7),
            offset_middle: ((offset & 0x00000000ffff0000) >> 16) as u16,
            offset_high: ((offset & 0xffffffff00000000) >> 32) as u32,
            reserved: 0,
        }
    }

    const fn default() -> Self {
        Self {
            offset_low: 0,
            segment_selector: 0,
            options_0: 0,
            options_1: 0,
            offset_middle: 0,
            offset_high: 0,
            reserved: 0,
        }
    }
}

#[repr(C, align(4096))]
struct Idt {
    division_error: IdtEntry,
    debug_exception: IdtEntry,
    nmi_interrupt: IdtEntry,
    breakpoint: IdtEntry,
    overflow: IdtEntry,
    bound_range_exceeded: IdtEntry,
    invalid_opcode: IdtEntry,
    device_not_avail: IdtEntry,
    double_fault: IdtEntry,
    _coprocessor_segment_overrun: IdtEntry,
    invalid_tss: IdtEntry,
    segment_not_present: IdtEntry,
    stack_segment_fault: IdtEntry,
    general_protection: IdtEntry,
    page_fault: IdtEntry,
    _intel_reserved_0: IdtEntry,
    math_fault: IdtEntry,
    alignment_check: IdtEntry,
    machine_check: IdtEntry,
    simd_fp_exception: IdtEntry,
    virtualization_exception: IdtEntry,
    control_protection_exception: IdtEntry,
    _intel_reserved_1: [IdtEntry; 10],
    user_define: [IdtEntry; 16],
}

impl Idt {
    const fn default() -> Self {
        Self {
            division_error: IdtEntry::default(),
            debug_exception: IdtEntry::default(),
            nmi_interrupt: IdtEntry::default(),
            breakpoint: IdtEntry::default(),
            overflow: IdtEntry::default(),
            bound_range_exceeded: IdtEntry::default(),
            invalid_opcode: IdtEntry::default(),
            device_not_avail: IdtEntry::default(),
            double_fault: IdtEntry::default(),
            _coprocessor_segment_overrun: IdtEntry::default(),
            invalid_tss: IdtEntry::default(),
            segment_not_present: IdtEntry::default(),
            stack_segment_fault: IdtEntry::default(),
            general_protection: IdtEntry::default(),
            page_fault: IdtEntry::default(),
            _intel_reserved_0: IdtEntry::default(),
            math_fault: IdtEntry::default(),
            alignment_check: IdtEntry::default(),
            machine_check: IdtEntry::default(),
            simd_fp_exception: IdtEntry::default(),
            virtualization_exception: IdtEntry::default(),
            control_protection_exception: IdtEntry::default(),
            _intel_reserved_1: [IdtEntry::default(); 10],
            user_define: [IdtEntry::default(); 16],
        }
    }
}

lazy_static! {
    static ref IDT: Idt = {
        let mut idt = Idt::default();

        idt.breakpoint = IdtEntry::new(
            breakpoint as u64,
            gdt::KERNEL_CODE_DESCRIPTOR,
            GateType::TrapGate,
            PrivilegeLevel::Ring0,
            true,
        );

        idt.double_fault = IdtEntry::new(
            double_fault as u64,
            gdt::KERNEL_CODE_DESCRIPTOR,
            GateType::TrapGate,
            PrivilegeLevel::Ring0,
            true,
        );

        idt.page_fault = IdtEntry::new(
            page_fault as u64,
            gdt::KERNEL_CODE_DESCRIPTOR,
            GateType::TrapGate,
            PrivilegeLevel::Ring0,
            true,
        );

        idt.user_define[0] = IdtEntry::new(
            timer as u64,
            gdt::KERNEL_CODE_DESCRIPTOR,
            GateType::TrapGate,
            PrivilegeLevel::Ring0,
            true,
        );

        idt
    };
}

#[repr(C, packed)]
struct Idtr {
    size: u16,
    ptr: u64,
}

pub unsafe fn load_idt() {
    let size = (size_of_val(&*IDT) - 1) as u16;
    let ptr = &*IDT as *const Idt as u64;
    let idtr = Idtr { size, ptr };
    unsafe {
        asm!(
            "lidt [{}]",
            in(reg) &idtr
        );
    }
}

#[derive(Debug)]
#[repr(C)]
struct InterruptionStackFrame {
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}

#[derive(Debug)]
#[repr(C)]
struct InterruptionStackFrameWithErrorCode {
    error_code: u64,
    rip: u64,
    cs: u64,
    rflfags: u64,
    rsp: u64,
    ss: u64,
}

// We do not use "x86-interrupt" call conventions for my preferences

macro_rules! make_interruption_handler {
    ($id: ident => $inner: ident) => {
        #[naked]
        unsafe extern "C" fn $id() {
            unsafe {
                naked_asm!(
                    "push rdi",
                    "mov rdi, rsp",
                    "add rdi, 8",
                    "push rax", "push rbx", "push rcx", "push rdx", "push rbp", "push rsi",
                    "push r8", "push r9", "push r10", "push r11", "push r12", "push r13", "push r14",
                    "push r15",
                    "call {0}",
                    "pop r15", "pop r14", "pop r13", "pop r12", "pop r11", "pop r10", "pop r9", "pop r8",
                    "pop rsi", "pop rbp", "pop rdx", "pop rcx", "pop rbx", "pop rax",
                    "pop rdi",
                    "iretq",
                    sym $inner,
                );
            }
        }
    };
    ($id: ident => $inner: ident with_error_code) => {
        #[naked]
        unsafe extern "C" fn $id() {
            unsafe {
                naked_asm!(
                    "push rdi",
                    "mov rdi, rsp",
                    "add rdi, 8",
                    "push rax", "push rbx", "push rcx", "push rdx", "push rbp", "push rsi",
                    "push r8", "push r9", "push r10", "push r11", "push r12", "push r13", "push r14",
                    "push r15",
                    "call {0}",
                    "pop r15", "pop r14", "pop r13", "pop r12", "pop r11", "pop r10", "pop r9", "pop r8",
                    "pop rsi", "pop rbp", "pop rdx", "pop rcx", "pop rbx", "pop rax",
                    "pop rdi",
                    "iretq 8",
                    sym $inner,
                );
            }
        }
    };
}

fn read_cr2() -> u64 {
    let cr2: u64;
    unsafe {
        asm!(
            "mov rax, cr2",
            out("rax") cr2
        );
    }
    cr2
}

make_interruption_handler!(breakpoint => breakpoint_inner);

extern "sysv64" fn breakpoint_inner(frame: &InterruptionStackFrame) -> () {
    trace!("Breakpoint reached at {:x}!", frame.rip);
}

make_interruption_handler!(double_fault => double_fault_inner with_error_code);

extern "sysv64" fn double_fault_inner(frame: &InterruptionStackFrameWithErrorCode) -> () {
    trace!("Double fault at {:x}!", frame.rip);
    panic!("Double fault");
}

make_interruption_handler!(page_fault => page_fault_inner with_error_code);

extern "sysv64" fn page_fault_inner(frame: &InterruptionStackFrameWithErrorCode) -> () {
    trace!(
        "Page fault at {:x} for accessing {:x}!",
        frame.rip,
        read_cr2()
    );
    panic!();
}

make_interruption_handler!(timer => timer_inner);

extern "sysv64" fn timer_inner(_frame: &InterruptionStackFrame) -> () {
    unsafe {
        send_eoi(0);
    }
}
