use user_std::io::syscall_create_pipe;

use crate::io::{self, BorrowedCursor, IoSlice, IoSliceMut};

use super::{fd::FileDesc, syscall_to_io_error};

pub struct AnonPipe(FileDesc);

pub fn anon_pipe() -> io::Result<(AnonPipe, AnonPipe)> {
    let (reader, writer) = unsafe { syscall_create_pipe().map_err(syscall_to_io_error)? };

    let reader = FileDesc::from_raw_fd(reader);
    let writer = FileDesc::from_raw_fd(writer);

    // TODO: set cloexec
    // reader.set_cloexec()?;
    // writer.set_cloexec()?;

    Ok((AnonPipe(reader), AnonPipe(writer)))
}

impl AnonPipe {
    pub fn into_inner(self) -> FileDesc {
        self.0
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }

    pub fn read_buf(&self, buf: BorrowedCursor<'_>) -> io::Result<()> {
        self.0.read_buf(buf)
    }

    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.0.read_vectored(bufs)
    }

    pub fn is_read_vectored(&self) -> bool {
        self.0.is_read_vectored()
    }

    pub fn read_to_end(&self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.0.read_to_end(buf)
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    pub fn write_vectored(&self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        self.0.write_vectored(bufs)
    }

    pub fn is_write_vectored(&self) -> bool {
        self.0.is_write_vectored()
    }
}

pub fn read2(_p1: AnonPipe, _v1: &mut Vec<u8>, _p2: AnonPipe, _v2: &mut Vec<u8>) -> io::Result<()> {
    todo!()
}
