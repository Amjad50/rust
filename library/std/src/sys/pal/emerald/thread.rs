use crate::ffi::CStr;
use crate::io;
use crate::num::NonZeroUsize;
use crate::time::Duration;

pub struct Thread(!);

pub const DEFAULT_MIN_STACK_SIZE: usize = 4096;

impl Thread {
    // unsafe: see thread::Builder::spawn_unchecked for safety requirements
    pub unsafe fn new(_stack: usize, _p: Box<dyn FnOnce()>) -> io::Result<Thread> {
        todo!("Thread::new")
    }

    pub fn yield_now() {
        // do nothing
    }

    pub fn set_name(_name: &CStr) {
        // nope
    }

    pub fn sleep(duration: Duration) {
        let secs = duration.as_secs();
        let nsecs = duration.subsec_nanos() as _;

        // For now, we don't have signals or interrupts. So the sleep will
        // be executed normally.
        unsafe {
            emerald_std::clock::sleep(secs, nsecs).expect("Failed to sleep");
        }
    }

    pub fn join(self) {
        self.0
    }
}

pub fn available_parallelism() -> io::Result<NonZeroUsize> {
    todo!("available_parallelism")
}
