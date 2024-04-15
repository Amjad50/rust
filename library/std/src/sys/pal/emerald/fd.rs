#![unstable(reason = "not public", issue = "none", feature = "fd")]

use core::cmp;

use crate::{
    io::{self, BorrowedCursor, IoSlice, IoSliceMut, Read},
    os::emerald::io::{AsFd, AsRawFd, BorrowedFd, FromRawFd, IntoRawFd, OwnedFd, RawFd},
    sys_common::{AsInner, FromInner, IntoInner},
};

use emerald_std::io::FileMeta;

use super::syscall_to_io_error;

// TODO: add `ownedFd` and other fd types to manage dropping them
#[derive(Debug)]
pub struct FileDesc(OwnedFd);

// The maximum read limit on most POSIX-like systems is `SSIZE_MAX`,
// with the man page quoting that if the count of bytes to read is
// greater than `SSIZE_MAX` the result is "unspecified".
//
// On macOS, however, apparently the 64-bit libc is either buggy or
// intentionally showing odd behavior by rejecting any read with a size
// larger than or equal to INT_MAX. To handle both of these the read
// size is capped on both platforms.
#[allow(dead_code)]
const READ_LIMIT: usize = isize::MAX as usize;

#[allow(dead_code)]
const fn max_iov() -> usize {
    16 // The minimum value required by POSIX.
}

impl FileDesc {
    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        // let ret = cvt(unsafe {
        //     libc::read(
        //         self.as_raw_fd(),
        //         buf.as_mut_ptr() as *mut libc::c_void,
        //         cmp::min(buf.len(), READ_LIMIT),
        //     )
        // })?;
        // Ok(ret as usize)
        let ret = unsafe {
            emerald_std::io::syscall_read(self.0.as_raw_fd(), buf).map_err(syscall_to_io_error)?
        };
        Ok(ret as usize)
    }

    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        io::default_read_vectored(|b| self.read(b), bufs)
    }

    #[inline]
    pub fn is_read_vectored(&self) -> bool {
        false
    }

    pub fn read_to_end(&self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let mut me = self;
        (&mut me).read_to_end(buf)
    }

    pub fn read_at(&self, _buf: &mut [u8], _offset: u64) -> io::Result<usize> {
        // unsafe {
        //     cvt(pread64(
        //         self.as_raw_fd(),
        //         buf.as_mut_ptr() as *mut libc::c_void,
        //         cmp::min(buf.len(), READ_LIMIT),
        //         offset as off64_t,
        //     ))
        //     .map(|n| n as usize)
        // }
        todo!()
    }

    pub fn read_buf(&self, mut cursor: BorrowedCursor<'_>) -> io::Result<()> {
        // Safety: `cursor` has `capcity` initialized bytes, so we can use them without issues
        let buf = unsafe {
            core::slice::from_raw_parts_mut(
                cursor.as_mut().as_mut_ptr() as *mut u8,
                cmp::min(cursor.capacity(), READ_LIMIT),
            )
        };
        let ret = unsafe {
            emerald_std::io::syscall_read(self.0.as_raw_fd(), buf).map_err(syscall_to_io_error)?
        };

        // Safety: `ret` bytes were written to the initialized portion of the buffer
        unsafe {
            cursor.advance_unchecked(ret as usize);
        }
        Ok(())
    }

    pub fn read_vectored_at(&self, bufs: &mut [IoSliceMut<'_>], offset: u64) -> io::Result<usize> {
        io::default_read_vectored(|b| self.read_at(b, offset), bufs)
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        // let ret = cvt(unsafe {
        //     libc::write(
        //         self.as_raw_fd(),
        //         buf.as_ptr() as *const libc::c_void,
        //         cmp::min(buf.len(), READ_LIMIT),
        //     )
        // })?;
        // Ok(ret as usize)
        let ret = unsafe {
            emerald_std::io::syscall_write(self.0.as_raw_fd(), buf).map_err(syscall_to_io_error)?
        };
        Ok(ret as usize)
    }

    pub fn write_vectored(&self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        io::default_write_vectored(|b| self.write(b), bufs)
    }

    #[inline]
    pub fn is_write_vectored(&self) -> bool {
        false
    }

    pub fn write_at(&self, _buf: &[u8], _offset: u64) -> io::Result<usize> {
        // #[cfg(not(any(
        //     all(target_os = "linux", not(target_env = "musl")),
        //     target_os = "android",
        //     target_os = "hurd"
        // )))]
        // use libc::pwrite as pwrite64;
        // #[cfg(any(
        //     all(target_os = "linux", not(target_env = "musl")),
        //     target_os = "android",
        //     target_os = "hurd"
        // ))]
        // use libc::pwrite64;

        // unsafe {
        //     cvt(pwrite64(
        //         self.as_raw_fd(),
        //         buf.as_ptr() as *const libc::c_void,
        //         cmp::min(buf.len(), READ_LIMIT),
        //         offset as off64_t,
        //     ))
        //     .map(|n| n as usize)
        // }
        todo!()
    }

    pub fn write_vectored_at(&self, bufs: &[IoSlice<'_>], offset: u64) -> io::Result<usize> {
        io::default_write_vectored(|b| self.write_at(b, offset), bufs)
    }

    #[cfg(not(any(
        target_env = "newlib",
        target_os = "solaris",
        target_os = "illumos",
        target_os = "emscripten",
        target_os = "fuchsia",
        target_os = "l4re",
        target_os = "linux",
        target_os = "haiku",
        target_os = "redox",
        target_os = "vxworks",
        target_os = "nto",
    )))]
    pub fn set_cloexec(&self) -> io::Result<()> {
        // unsafe {
        //     cvt(libc::ioctl(self.as_raw_fd(), libc::FIOCLEX))?;
        //     Ok(())
        // }
        todo!()
    }
    #[cfg(any(
        all(
            target_env = "newlib",
            not(any(target_os = "espidf", target_os = "horizon", target_os = "vita"))
        ),
        target_os = "solaris",
        target_os = "illumos",
        target_os = "emscripten",
        target_os = "fuchsia",
        target_os = "l4re",
        target_os = "linux",
        target_os = "haiku",
        target_os = "redox",
        target_os = "vxworks",
        target_os = "nto",
    ))]
    pub fn set_cloexec(&self) -> io::Result<()> {
        unsafe {
            let previous = cvt(libc::fcntl(self.as_raw_fd(), libc::F_GETFD))?;
            let new = previous | libc::FD_CLOEXEC;
            if new != previous {
                cvt(libc::fcntl(self.as_raw_fd(), libc::F_SETFD, new))?;
            }
            Ok(())
        }
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        let blocking_mode = if nonblocking {
            emerald_std::io::BlockingMode::None
        } else {
            todo!("Not sure which mode to put here")
            // emerald_std::io::BlockingMode::Line
            // emerald_std::io::BlockingMode::Block(1)
        };

        unsafe {
            emerald_std::io::syscall_set_file_meta(
                self.as_raw_fd(),
                FileMeta::BlockingMode(blocking_mode),
            )
            .map_err(syscall_to_io_error)?
        }

        Ok(())
    }

    #[inline]
    pub fn duplicate(&self) -> io::Result<FileDesc> {
        // Ok(Self(self.0.try_clone()?))
        todo!()
    }
}

impl<'a> Read for &'a FileDesc {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (**self).read(buf)
    }

    fn read_buf(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        (**self).read_buf(cursor)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        (**self).read_vectored(bufs)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        (**self).is_read_vectored()
    }
}

impl AsInner<OwnedFd> for FileDesc {
    #[inline]
    fn as_inner(&self) -> &OwnedFd {
        &self.0
    }
}

impl IntoInner<OwnedFd> for FileDesc {
    fn into_inner(self) -> OwnedFd {
        self.0
    }
}

impl FromInner<OwnedFd> for FileDesc {
    fn from_inner(owned_fd: OwnedFd) -> Self {
        Self(owned_fd)
    }
}

impl AsFd for FileDesc {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl AsRawFd for FileDesc {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl IntoRawFd for FileDesc {
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

impl FromRawFd for FileDesc {
    unsafe fn from_raw_fd(raw_fd: RawFd) -> Self {
        Self(unsafe { FromRawFd::from_raw_fd(raw_fd) })
    }
}
