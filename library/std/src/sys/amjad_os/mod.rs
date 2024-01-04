#![deny(unsafe_op_in_unsafe_fn)]

pub mod alloc;
pub mod args;
#[path = "../unix/cmath.rs"]
pub mod cmath;
pub mod env;
pub mod fd;
pub mod fs;
pub mod io;
pub mod locks;
pub mod net;
pub mod once;
pub mod os;
#[path = "../unix/os_str.rs"]
pub mod os_str;
#[path = "../unix/path.rs"]
pub mod path;
pub mod pipe;
pub mod process;
pub mod stdio;
pub mod thread;
#[cfg(target_thread_local)]
pub mod thread_local_dtor;
pub mod thread_local_key;
pub mod thread_parking;
pub mod time;

mod common;
pub use common::*;

use user_std::SyscallError;

fn syscall_to_io_error(e: SyscallError) -> crate::io::Error {
    match e {
        SyscallError::EndOfFile => {
            crate::io::Error::new(crate::io::ErrorKind::UnexpectedEof, "Unexpected end of file")
        }
        SyscallError::FileNotFound => {
            crate::io::Error::new(crate::io::ErrorKind::NotFound, "File not found")
        }
        SyscallError::CouldNotOpenFile => {
            crate::io::Error::new(crate::io::ErrorKind::NotFound, "Could not open file")
        }
        SyscallError::InvalidFileIndex => {
            crate::io::Error::new(crate::io::ErrorKind::NotFound, "Invalid file index")
        }
        SyscallError::CouldNotWriteToFile => {
            crate::io::Error::new(crate::io::ErrorKind::PermissionDenied, "Could not write to file")
        }
        SyscallError::CouldNotReadFromFile => crate::io::Error::new(
            crate::io::ErrorKind::PermissionDenied,
            "Could not read from file",
        ),
        SyscallError::InvalidArgument(_, _, _, _, _, _, _) => {
            // TODO: use args
            crate::io::Error::new(crate::io::ErrorKind::InvalidInput, "Invalid argument")
        }
        // should never happen
        SyscallError::SyscallNotFound | SyscallError::InvalidErrorCode(_) => unreachable!(),
        // not applicable
        SyscallError::CouldNotLoadElf
        | SyscallError::CouldNotAllocateProcess
        | SyscallError::HeapRangesExceeded => unreachable!(),
    }
}
