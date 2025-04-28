use super::definitions::PhysAddress;

pub unsafe fn borrow_from_phys_addr_mut<T>(addr: PhysAddress) -> &'static mut T {
    let u: usize = addr.into();
    unsafe { &mut *(u as *mut T) }
}
