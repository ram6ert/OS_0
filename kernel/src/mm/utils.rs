use super::definitions::{PHYSICAL_MAP_BEGIN, PhysAddress};

pub fn calculate_phys_addr_from_pptr<T>(addr: *mut T) -> PhysAddress {
    let u = addr as usize;
    PhysAddress::new(u - PHYSICAL_MAP_BEGIN)
}

pub fn calculate_pptr_from_phys_addr<T>(addr: PhysAddress) -> *mut T {
    let u = addr.as_usize();
    (u + PHYSICAL_MAP_BEGIN) as *mut T
}

pub unsafe fn borrow_from_phys_addr_mut<T>(addr: PhysAddress) -> &'static mut T {
    unsafe { &mut *calculate_pptr_from_phys_addr::<T>(addr) }
}
