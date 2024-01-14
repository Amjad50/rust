//! amjad_os-specific extensions to primitives in the [`std::process`] module.
//!
//! [`std::process`]: crate::process

#![stable(feature = "rust1", since = "1.0.0")]

use crate::{
    process,
    sealed::Sealed,
    sys_common::{AsInner, FromInner},
};

/// amjad_os extension to [`process::ExitStatus`] that is based on unix as below
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
