//! emerald-specific extensions to primitives in the [`std::process`] module.
//!
//! [`std::process`]: crate::process

#![stable(feature = "rust1", since = "1.0.0")]

use crate::os::emerald::io::{AsFd, AsRawFd, BorrowedFd, FromRawFd, IntoRawFd, OwnedFd, RawFd};
use crate::sealed::Sealed;
use crate::sys_common::{AsInner, FromInner, IntoInner};
use crate::{process, sys};

/// `emerald` extension to [`process::ExitStatus`] that is based on unix as below
/// ...
///
/// Unix-specific extensions to [`process::ExitStatus`] and
/// [`ExitStatusError`](process::ExitStatusError).
///
/// On Unix, `ExitStatus` **does not necessarily represent an exit status**, as
/// passed to the `_exit` system call or returned by
/// [`ExitStatus::code()`](crate::process::ExitStatus::code).  It represents **any wait status**
/// as returned by one of the `wait` family of system
/// calls.
///
/// A Unix wait status (a Rust `ExitStatus`) can represent a Unix exit status, but can also
/// represent other kinds of process event.
///
/// This trait is sealed: it cannot be implemented outside the standard library.
/// This is so that future additional methods are not breaking changes.
#[stable(feature = "rust1", since = "1.0.0")]
pub trait ExitStatusExt: Sealed {
    /// Creates a new `ExitStatus` or `ExitStatusError` from the raw underlying integer status
    /// value from `wait`
    ///
    /// The value should be a **wait status, not an exit status**.
    ///
    /// # Panics
    ///
    /// Panics on an attempt to make an `ExitStatusError` from a wait status of `0`.
    ///
    /// Making an `ExitStatus` always succeeds and never panics.
    #[stable(feature = "exit_status_from", since = "1.12.0")]
    fn from_raw(raw: i32) -> Self;

    /// Returns the underlying raw `wait` status.
    ///
    /// The returned integer is a **wait status, not an exit status**.
    #[stable(feature = "unix_process_wait_more", since = "1.58.0")]
    fn into_raw(self) -> i32;
}

#[stable(feature = "rust1", since = "1.0.0")]
impl ExitStatusExt for process::ExitStatus {
    fn from_raw(raw: i32) -> Self {
        process::ExitStatus::from_inner(From::from(raw))
    }

    fn into_raw(self) -> i32 {
        self.as_inner().into_raw().into()
    }
}

#[unstable(feature = "exit_status_error", issue = "84908")]
impl ExitStatusExt for process::ExitStatusError {
    fn from_raw(raw: i32) -> Self {
        process::ExitStatus::from_raw(raw)
            .exit_ok()
            .expect_err("<ExitStatusError as ExitStatusExt>::from_raw(0) but zero is not an error")
    }

    fn into_raw(self) -> i32 {
        self.into_status().into_raw()
    }
}

#[stable(feature = "process_extensions", since = "1.2.0")]
impl FromRawFd for process::Stdio {
    #[inline]
    unsafe fn from_raw_fd(fd: RawFd) -> process::Stdio {
        let fd = sys::fd::FileDesc::from_raw_fd(fd);
        let io = sys::process::Stdio::Fd(fd);
        process::Stdio::from_inner(io)
    }
}

#[stable(feature = "io_safety", since = "1.63.0")]
impl From<OwnedFd> for process::Stdio {
    /// Takes ownership of a file descriptor and returns a [`Stdio`](process::Stdio)
    /// that can attach a stream to it.
    #[inline]
    fn from(fd: OwnedFd) -> process::Stdio {
        let fd = sys::fd::FileDesc::from_inner(fd);
        let io = sys::process::Stdio::Fd(fd);
        process::Stdio::from_inner(io)
    }
}

#[stable(feature = "process_extensions", since = "1.2.0")]
impl AsRawFd for process::ChildStdin {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.as_inner().as_raw_fd()
    }
}

#[stable(feature = "process_extensions", since = "1.2.0")]
impl AsRawFd for process::ChildStdout {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.as_inner().as_raw_fd()
    }
}

#[stable(feature = "process_extensions", since = "1.2.0")]
impl AsRawFd for process::ChildStderr {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.as_inner().as_raw_fd()
    }
}

#[stable(feature = "into_raw_os", since = "1.4.0")]
impl IntoRawFd for process::ChildStdin {
    #[inline]
    fn into_raw_fd(self) -> RawFd {
        self.into_inner().into_inner().into_raw_fd()
    }
}

#[stable(feature = "into_raw_os", since = "1.4.0")]
impl IntoRawFd for process::ChildStdout {
    #[inline]
    fn into_raw_fd(self) -> RawFd {
        self.into_inner().into_inner().into_raw_fd()
    }
}

#[stable(feature = "into_raw_os", since = "1.4.0")]
impl IntoRawFd for process::ChildStderr {
    #[inline]
    fn into_raw_fd(self) -> RawFd {
        self.into_inner().into_inner().into_raw_fd()
    }
}

#[stable(feature = "io_safety", since = "1.63.0")]
impl AsFd for crate::process::ChildStdin {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.as_inner().as_fd()
    }
}

#[stable(feature = "io_safety", since = "1.63.0")]
impl From<crate::process::ChildStdin> for OwnedFd {
    /// Takes ownership of a [`ChildStdin`](crate::process::ChildStdin)'s file descriptor.
    #[inline]
    fn from(child_stdin: crate::process::ChildStdin) -> OwnedFd {
        child_stdin.into_inner().into_inner().into_inner()
    }
}

/// Creates a `ChildStdin` from the provided `OwnedFd`.
///
/// The provided file descriptor must point to a pipe
/// with the `CLOEXEC` flag set.
#[stable(feature = "child_stream_from_fd", since = "1.74.0")]
impl From<OwnedFd> for process::ChildStdin {
    #[inline]
    fn from(fd: OwnedFd) -> process::ChildStdin {
        let fd = sys::fd::FileDesc::from_inner(fd);
        let pipe = sys::pipe::AnonPipe::from_inner(fd);
        process::ChildStdin::from_inner(pipe)
    }
}

#[stable(feature = "io_safety", since = "1.63.0")]
impl AsFd for crate::process::ChildStdout {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.as_inner().as_fd()
    }
}

#[stable(feature = "io_safety", since = "1.63.0")]
impl From<crate::process::ChildStdout> for OwnedFd {
    /// Takes ownership of a [`ChildStdout`](crate::process::ChildStdout)'s file descriptor.
    #[inline]
    fn from(child_stdout: crate::process::ChildStdout) -> OwnedFd {
        child_stdout.into_inner().into_inner().into_inner()
    }
}

/// Creates a `ChildStdout` from the provided `OwnedFd`.
///
/// The provided file descriptor must point to a pipe
/// with the `CLOEXEC` flag set.
#[stable(feature = "child_stream_from_fd", since = "1.74.0")]
impl From<OwnedFd> for process::ChildStdout {
    #[inline]
    fn from(fd: OwnedFd) -> process::ChildStdout {
        let fd = sys::fd::FileDesc::from_inner(fd);
        let pipe = sys::pipe::AnonPipe::from_inner(fd);
        process::ChildStdout::from_inner(pipe)
    }
}

#[stable(feature = "io_safety", since = "1.63.0")]
impl AsFd for crate::process::ChildStderr {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.as_inner().as_fd()
    }
}

#[stable(feature = "io_safety", since = "1.63.0")]
impl From<crate::process::ChildStderr> for OwnedFd {
    /// Takes ownership of a [`ChildStderr`](crate::process::ChildStderr)'s file descriptor.
    #[inline]
    fn from(child_stderr: crate::process::ChildStderr) -> OwnedFd {
        child_stderr.into_inner().into_inner().into_inner()
    }
}

/// Creates a `ChildStderr` from the provided `OwnedFd`.
///
/// The provided file descriptor must point to a pipe
/// with the `CLOEXEC` flag set.
#[stable(feature = "child_stream_from_fd", since = "1.74.0")]
impl From<OwnedFd> for process::ChildStderr {
    #[inline]
    fn from(fd: OwnedFd) -> process::ChildStderr {
        let fd = sys::fd::FileDesc::from_inner(fd);
        let pipe = sys::pipe::AnonPipe::from_inner(fd);
        process::ChildStderr::from_inner(pipe)
    }
}
