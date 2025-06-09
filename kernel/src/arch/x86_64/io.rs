use core::arch::asm;

#[inline(always)]
pub unsafe fn in8(port: u16) -> u8 {
    let mut result: u8;
    unsafe {
        asm!("in al, dx", in("dx") port, out("al") result);
    }
    result
}

#[inline(always)]
pub unsafe fn out8(port: u16, data: u8) {
    unsafe { asm!("out dx, al", in("dx") port, in("al") data) }
}
