use core::sync::atomic::{AtomicUsize, Ordering};

pub type Key = usize;

// FIXME: this is only for 1 single thread at a time
static mut MAP: [*mut u8; 256] = [core::ptr::null_mut(); 256];
static MAP_INDEX: AtomicUsize = AtomicUsize::new(0);

#[inline]
pub unsafe fn create(_dtor: Option<unsafe extern "C" fn(*mut u8)>) -> Key {
    // panic!("should not be used on this target");
    let index = MAP_INDEX.fetch_add(1, Ordering::SeqCst);
    index
}

#[inline]
pub unsafe fn set(key: Key, value: *mut u8) {
    // panic!("should not be used on this target");
    unsafe {
        MAP[key] = value;
    }
}

#[inline]
pub unsafe fn get(key: Key) -> *mut u8 {
    // panic!("should not be used on this target");
    unsafe { MAP[key] }
}

#[inline]
pub unsafe fn destroy(key: Key) {
    // panic!("should not be used on this target");
    unsafe {
        MAP[key] = core::ptr::null_mut();
    }
}
