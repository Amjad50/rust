use core::{io::BorrowedCursor, mem::ManuallyDrop};

use crate::{
    io::{self, IoSlice, IoSliceMut},
    os::amjad_os::io::FromRawFd,
};

use super::fd::FileDesc;

pub struct Stdin(());
pub struct Stdout(());
pub struct Stderr(());

impl Stdin {
    pub const fn new() -> Stdin {
        Stdin(())
    }
}

impl io::Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        ManuallyDrop::new(unsafe { FileDesc::from_raw_fd(user_std::io::FD_STDIN) }).read(buf)
    }

    fn read_buf(&mut self, buf: BorrowedCursor<'_>) -> io::Result<()> {
        ManuallyDrop::new(unsafe { FileDesc::from_raw_fd(user_std::io::FD_STDIN) }).read_buf(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        ManuallyDrop::new(unsafe { FileDesc::from_raw_fd(user_std::io::FD_STDIN) })
            .read_vectored(bufs)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        ManuallyDrop::new(unsafe { FileDesc::from_raw_fd(user_std::io::FD_STDIN) })
            .is_read_vectored()
    }
}

impl Stdout {
    pub const fn new() -> Stdout {
        Stdout(())
    }
}

impl io::Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        ManuallyDrop::new(unsafe { FileDesc::from_raw_fd(user_std::io::FD_STDOUT) }).write(buf)
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        ManuallyDrop::new(unsafe { FileDesc::from_raw_fd(user_std::io::FD_STDOUT) })
            .write_vectored(bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        ManuallyDrop::new(unsafe { FileDesc::from_raw_fd(user_std::io::FD_STDOUT) })
            .is_write_vectored()
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Stderr {
    pub const fn new() -> Stderr {
        Stderr(())
    }
}

impl io::Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        ManuallyDrop::new(unsafe { FileDesc::from_raw_fd(user_std::io::FD_STDERR) }).write(buf)
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        ManuallyDrop::new(unsafe { FileDesc::from_raw_fd(user_std::io::FD_STDERR) })
            .write_vectored(bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        ManuallyDrop::new(unsafe { FileDesc::from_raw_fd(user_std::io::FD_STDERR) })
            .is_write_vectored()
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub const STDIN_BUF_SIZE: usize = crate::sys_common::io::DEFAULT_BUF_SIZE;

pub fn is_ebadf(err: &io::Error) -> bool {
    match err.kind() {
        io::ErrorKind::UnexpectedEof | io::ErrorKind::PermissionDenied => true,
        _ => false,
    }
}

pub fn panic_output() -> Option<impl io::Write> {
    Some(Stderr::new())
}
