#![deny(unsafe_op_in_unsafe_fn)]

pub mod alloc;
pub mod args;
pub mod env;
pub mod fd;
pub mod fs;
pub mod io;
pub mod net;
pub mod os;
pub mod pipe;
pub mod process;
pub mod stdio;
pub mod thread;
#[cfg(target_thread_local)]
pub mod thread_local_dtor;
pub mod thread_local_key;
pub mod time;

mod common;
pub use common::*;

use emerald_std::SyscallError;

fn syscall_to_io_error(e: SyscallError) -> crate::io::Error {
    match e {
        SyscallError::PidNotFound => crate::io::Error::new(
            crate::io::ErrorKind::NotFound,
            "Process with given pid not found",
        ),
        SyscallError::ProcessStillRunning => crate::io::Error::new(
            crate::io::ErrorKind::Other,
            "Process with given pid is still running",
        ),
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
        SyscallError::IsNotDirectory => {
            crate::io::Error::new(crate::io::ErrorKind::NotADirectory, "Is not a directory")
        }
        SyscallError::IsDirectory => {
            crate::io::Error::new(crate::io::ErrorKind::IsADirectory, "Is a directory")
        }
        SyscallError::AlreadyExists => {
            crate::io::Error::new(crate::io::ErrorKind::AlreadyExists, "Already exists")
        }
        SyscallError::OperationNotSupported => {
            crate::io::Error::new(crate::io::ErrorKind::Unsupported, "Not supported")
        }
        SyscallError::InvalidArgument(arg1, arg2, arg3, arg4, arg5, arg6, arg7) => {
            let errors = [arg1, arg2, arg3, arg4, arg5, arg6, arg7];

            let mut error_str = String::new();

            errors.iter().take_while(|e| e.is_some()).enumerate().for_each(|(i, e)| {
                if i != 0 {
                    error_str.push_str(", ");
                }
                error_str.push_str(&format!("Arg{i}: {:?}", e.unwrap()));
            });

            crate::io::Error::new(crate::io::ErrorKind::InvalidInput, error_str)
        }
        // should never happen
        SyscallError::SyscallNotFound | SyscallError::InvalidError => unreachable!(),
        // not applicable
        SyscallError::CouldNotLoadElf
        | SyscallError::CouldNotAllocateProcess
        | SyscallError::HeapRangesExceeded => unreachable!(),
        _ => crate::io::Error::new(crate::io::ErrorKind::Other, "Unknown error"),
    }
}
