use emerald_std::SyscallError;

use crate::error::Error as StdError;
use crate::ffi::{OsStr, OsString};
use crate::fmt;
use crate::io;
use crate::marker::PhantomData;
use crate::os::emerald::prelude::OsStringExt;
use crate::path::{self, PathBuf};
use crate::ptr::addr_of;
use crate::sys::common::small_c_string::run_path_with_cstr;
use crate::sys::pal::emerald::syscall_to_io_error;

#[cfg(not(test))]
#[cfg(feature = "panic_unwind")]
mod eh_unwinding {
    pub(crate) struct EhFrameFinder(usize /* eh_frame */);
    pub(crate) static mut EH_FRAME_SETTINGS: EhFrameFinder = EhFrameFinder(0);
    impl EhFrameFinder {
        pub(crate) fn init(&mut self, eh_frame: usize) {
            self.0 = eh_frame;
        }
    }
    unsafe impl unwind::EhFrameFinder for EhFrameFinder {
        fn find(&self, _pc: usize) -> Option<unwind::FrameInfo> {
            Some(unwind::FrameInfo {
                text_base: None,
                kind: unwind::FrameInfoKind::EhFrame(self.0),
            })
        }
    }
}

// This function is needed by the panic runtime. The symbol is named in
// pre-link args for the target specification, so keep that in sync.
#[no_mangle]
pub extern "C" fn __rust_abort() -> ! {
    super::os::exit(0xFF);
}
extern "C" {
    fn main(argc: isize, argv: *const *const u8) -> i32;
}

#[no_mangle]
pub extern "C" fn _start(argc: isize, argv: *const *const u8) -> ! {
    #[cfg(not(test))]
    #[cfg(feature = "panic_unwind")]
    unsafe {
        eh_unwinding::EH_FRAME_SETTINGS
            .init(emerald_std::process::process_metadata().eh_frame_address);
        unwind::set_custom_eh_frame_finder(&*addr_of!(eh_unwinding::EH_FRAME_SETTINGS)).ok();
    }
    exit(unsafe { main(argc, argv) });
}

pub fn errno() -> i32 {
    0
}

pub fn error_string(_errno: i32) -> String {
    "operation successful".to_string()
}

pub fn getcwd() -> io::Result<PathBuf> {
    let mut buf = Vec::with_capacity(512);
    loop {
        unsafe {
            // Safety: the size is equal to the capacity, I'm setting the length so we can access the slice
            //         its safe to use since the data will be overwritten by getcwd
            buf.set_len(buf.capacity());
            match emerald_std::io::syscall_get_cwd(&mut buf) {
                Ok(len) => {
                    // Safety: forcing the length back to the safe space
                    buf.set_len(len);
                    buf.shrink_to_fit();
                    return Ok(PathBuf::from(OsString::from_vec(buf)));
                }
                Err(SyscallError::BufferTooSmall) => {
                    // Trigger the internal buffer resizing logic of `Vec` by requiring
                    // more space than the current capacity.
                    // at this stage we have already set the length to the capacity
                    buf.reserve(1);
                }
                Err(e) => return Err(syscall_to_io_error(e)),
            }
        }
    }
}

pub fn chdir(p: &path::Path) -> io::Result<()> {
    run_path_with_cstr(p, |p| unsafe {
        emerald_std::io::syscall_chdir(p).map_err(syscall_to_io_error)
    })
}

pub struct SplitPaths<'a>(!, PhantomData<&'a ()>);

pub fn split_paths(_unparsed: &OsStr) -> SplitPaths<'_> {
    todo!("split_paths")
}

impl<'a> Iterator for SplitPaths<'a> {
    type Item = PathBuf;
    fn next(&mut self) -> Option<PathBuf> {
        self.0
    }
}

#[derive(Debug)]
pub struct JoinPathsError;

pub fn join_paths<I, T>(_paths: I) -> Result<OsString, JoinPathsError>
where
    I: Iterator<Item = T>,
    T: AsRef<OsStr>,
{
    Err(JoinPathsError)
}

impl fmt::Display for JoinPathsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "not supported on this platform yet".fmt(f)
    }
}

impl StdError for JoinPathsError {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        "not supported on this platform yet"
    }
}

pub fn current_exe() -> io::Result<PathBuf> {
    use crate::env;
    use crate::io::ErrorKind;

    let exe_path = env::args().next().ok_or(io::const_io_error!(
        ErrorKind::Uncategorized,
        "an executable path was not found because no arguments were provided through argv"
    ))?;
    let path = PathBuf::from(exe_path);

    // Prepend the current working directory to the path if it's not absolute.
    // TODO: `is_absolute` is broken for non-unix since it relies on `cfgs` which I don't want to modify,
    //       I'm opening a PR to fix that, then this will be changed to `is_absolute` but doesn't change much anyway
    if !path.has_root() { getcwd().map(|cwd| cwd.join(path)) } else { Ok(path) }
}

pub struct Env(!);

impl Env {
    // FIXME(https://github.com/rust-lang/rust/issues/114583): Remove this when <OsStr as Debug>::fmt matches <str as Debug>::fmt.
    pub fn str_debug(&self) -> impl fmt::Debug + '_ {
        let Self(inner) = self;
        match *inner {}
    }
}

impl fmt::Debug for Env {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(inner) = self;
        match *inner {}
    }
}

impl Iterator for Env {
    type Item = (OsString, OsString);
    fn next(&mut self) -> Option<(OsString, OsString)> {
        let Self(inner) = self;
        match *inner {}
    }
}

pub fn env() -> Env {
    panic!("not supported on this platform")
}

pub fn getenv(str: &OsStr) -> Option<OsString> {
    if str == "RUST_BACKTRACE" {
        return Some("full".into());
    }
    None
}

pub fn setenv(_: &OsStr, _: &OsStr) -> io::Result<()> {
    Err(io::const_io_error!(io::ErrorKind::Unsupported, "cannot set env vars on this platform"))
}

pub fn unsetenv(_: &OsStr) -> io::Result<()> {
    Err(io::const_io_error!(io::ErrorKind::Unsupported, "cannot unset env vars on this platform"))
}

pub fn temp_dir() -> PathBuf {
    panic!("no filesystem on this platform")
}

pub fn home_dir() -> Option<PathBuf> {
    None
}

pub fn exit(code: i32) -> ! {
    unsafe { emerald_std::process::exit(code) }
}

pub fn getpid() -> u32 {
    panic!("no pids on this platform")
}
