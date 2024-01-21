use crate::mem;

use crate::os::amjad_os::io::{AsFd, AsRawFd};
use crate::sys::amjad_os::syscall_to_io_error;
use user_std::io::FileMeta;

#[derive(Copy, Clone)]
pub struct IoSlice<'a>(&'a [u8]);

impl<'a> IoSlice<'a> {
    #[inline]
    pub fn new(buf: &'a [u8]) -> IoSlice<'a> {
        IoSlice(buf)
    }

    #[inline]
    pub fn advance(&mut self, n: usize) {
        self.0 = &self.0[n..]
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        self.0
    }
}

pub struct IoSliceMut<'a>(&'a mut [u8]);

impl<'a> IoSliceMut<'a> {
    #[inline]
    pub fn new(buf: &'a mut [u8]) -> IoSliceMut<'a> {
        IoSliceMut(buf)
    }

    #[inline]
    pub fn advance(&mut self, n: usize) {
        let slice = mem::take(&mut self.0);
        let (_, remaining) = slice.split_at_mut(n);
        self.0 = remaining;
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        self.0
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.0
    }
}

pub fn is_terminal(file: &impl AsFd) -> bool {
    let mut meta = FileMeta::IsTerminal(false);
    unsafe {
        user_std::io::syscall_get_file_meta(file.as_fd().as_raw_fd(), &mut meta)
            .map_err(syscall_to_io_error)
            .expect("syscall_get_file_meta failed");
    }

    match meta {
        FileMeta::IsTerminal(is_terminal) => is_terminal,
        _ => unreachable!(),
    }
}
