use super::args;

// SAFETY: must be called only once during runtime initialization.
// NOTE: this is not guaranteed to run, for example when Rust code is called externally.
pub unsafe fn init(argc: isize, argv: *const *const u8, _sigpipe: u8) {
    unsafe { args::init(argc, argv) }
}

// SAFETY: must be called only once during runtime cleanup.
// NOTE: this is not guaranteed to run, for example when the program aborts.
pub unsafe fn cleanup() {}

pub fn is_interrupted(_code: i32) -> bool {
    false
}

pub fn decode_error_kind(_code: i32) -> crate::io::ErrorKind {
    crate::io::ErrorKind::Uncategorized
}

pub fn abort_internal() -> ! {
    super::os::__rust_abort()
}

pub fn hashmap_random_keys() -> (u64, u64) {
    (1, 2)
}
