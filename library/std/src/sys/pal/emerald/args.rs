use core::ffi::{c_char, CStr};
use core::slice;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::ffi::OsString;
use crate::fmt;

// A pointer to `Vec<OsString>`.
static ARGS: AtomicUsize = AtomicUsize::new(0);
type ArgsStore = Vec<OsString>;

/// One-time global initialization.
pub unsafe fn init(argc: isize, argv: *const *const u8) {
    let argc = if argv.is_null() { 0 } else { argc };

    let args = (0..argc)
        .map(|i| unsafe {
            let cstr = CStr::from_ptr(*argv.offset(i) as *const c_char);
            OsString::from_encoded_bytes_unchecked(cstr.to_bytes().to_vec())
        })
        .collect::<ArgsStore>();
    ARGS.store(Box::into_raw(Box::new(args)) as _, Ordering::Relaxed);
}

pub struct Args(slice::Iter<'static, OsString>);

pub fn args() -> Args {
    let ptr = core::ptr::with_exposed_provenance(ARGS.load(Ordering::Relaxed)) as *const ArgsStore;
    let args = unsafe { ptr.as_ref() };
    if let Some(args) = args { Args(args.iter()) } else { Args([].iter()) }
}

impl fmt::Debug for Args {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.as_slice().fmt(f)
    }
}

impl Iterator for Args {
    type Item = OsString;
    fn next(&mut self) -> Option<OsString> {
        self.0.next().cloned()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl ExactSizeIterator for Args {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl DoubleEndedIterator for Args {
    fn next_back(&mut self) -> Option<OsString> {
        self.0.next_back().cloned()
    }
}
