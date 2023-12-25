pub type Key = usize;

// FIXME: this is only for 1 single thread at a time
static mut DATA: *mut u8 = core::ptr::null_mut();

#[inline]
pub unsafe fn create(_dtor: Option<unsafe extern "C" fn(*mut u8)>) -> Key {
    // panic!("should not be used on this target");
    1
}

#[inline]
pub unsafe fn set(_key: Key, _value: *mut u8) {
    // panic!("should not be used on this target");
    unsafe { DATA = _value };
}

#[inline]
pub unsafe fn get(_key: Key) -> *mut u8 {
    // panic!("should not be used on this target");
    unsafe { DATA }
}

#[inline]
pub unsafe fn destroy(_key: Key) {
    // panic!("should not be used on this target");
}
