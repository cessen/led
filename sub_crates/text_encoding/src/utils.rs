use core::mem::transmute;

#[inline(always)]
pub(crate) fn to_big_endian_u16(n: u16) -> [u8; 2] {
    let ptr = unsafe { transmute::<*const u16, *const u8>(&n as *const u16) };
    if cfg!(target_endian = "little") {
        unsafe { [*ptr.offset(1), *ptr] }
    } else {
        unsafe { [*ptr, *ptr.offset(1)] }
    }
}

#[inline(always)]
pub(crate) fn from_big_endian_u16(n: [u8; 2]) -> u16 {
    let mut x: u16 = 0;
    let ptr = unsafe { transmute::<*mut u16, *mut u8>(&mut x as *mut u16) };
    if cfg!(target_endian = "little") {
        unsafe {
            *ptr = n[1];
            *ptr.offset(1) = n[0];
        }
    } else {
        unsafe {
            *ptr = n[0];
            *ptr.offset(1) = n[1];
        }
    }
    x
}

#[inline(always)]
pub(crate) fn to_little_endian_u16(n: u16) -> [u8; 2] {
    let ptr = unsafe { transmute::<*const u16, *const u8>(&n as *const u16) };
    if cfg!(target_endian = "little") {
        unsafe { [*ptr, *ptr.offset(1)] }
    } else {
        unsafe { [*ptr.offset(1), *ptr] }
    }
}

#[inline(always)]
pub(crate) fn from_little_endian_u16(n: [u8; 2]) -> u16 {
    let mut x: u16 = 0;
    let ptr = unsafe { transmute::<*mut u16, *mut u8>(&mut x as *mut u16) };
    if cfg!(target_endian = "little") {
        unsafe {
            *ptr = n[0];
            *ptr.offset(1) = n[1];
        }
    } else {
        unsafe {
            *ptr = n[1];
            *ptr.offset(1) = n[0];
        }
    }
    x
}

#[inline(always)]
pub(crate) fn to_big_endian_u32(n: u32) -> [u8; 4] {
    let ptr = unsafe { transmute::<*const u32, *const u8>(&n as *const u32) };
    if cfg!(target_endian = "little") {
        unsafe { [*ptr.offset(3), *ptr.offset(2), *ptr.offset(1), *ptr] }
    } else {
        unsafe { [*ptr, *ptr.offset(1), *ptr.offset(2), *ptr.offset(3)] }
    }
}

#[inline(always)]
pub(crate) fn from_big_endian_u32(n: [u8; 4]) -> u32 {
    let mut x: u32 = 0;
    let ptr = unsafe { transmute::<*mut u32, *mut u8>(&mut x as *mut u32) };
    if cfg!(target_endian = "little") {
        unsafe {
            *ptr = n[3];
            *ptr.offset(1) = n[2];
            *ptr.offset(2) = n[1];
            *ptr.offset(3) = n[0];
        }
    } else {
        unsafe {
            *ptr = n[0];
            *ptr.offset(1) = n[1];
            *ptr.offset(2) = n[2];
            *ptr.offset(3) = n[3];
        }
    }
    x
}

#[inline(always)]
pub(crate) fn to_little_endian_u32(n: u32) -> [u8; 4] {
    let ptr = unsafe { transmute::<*const u32, *const u8>(&n as *const u32) };
    if cfg!(target_endian = "little") {
        unsafe { [*ptr, *ptr.offset(1), *ptr.offset(2), *ptr.offset(3)] }
    } else {
        unsafe { [*ptr.offset(3), *ptr.offset(2), *ptr.offset(1), *ptr] }
    }
}

#[inline(always)]
pub(crate) fn from_little_endian_u32(n: [u8; 4]) -> u32 {
    let mut x: u32 = 0;
    let ptr = unsafe { transmute::<*mut u32, *mut u8>(&mut x as *mut u32) };
    if cfg!(target_endian = "little") {
        unsafe {
            *ptr = n[0];
            *ptr.offset(1) = n[1];
            *ptr.offset(2) = n[2];
            *ptr.offset(3) = n[3];
        }
    } else {
        unsafe {
            *ptr = n[3];
            *ptr.offset(1) = n[2];
            *ptr.offset(2) = n[1];
            *ptr.offset(3) = n[0];
        }
    }
    x
}
