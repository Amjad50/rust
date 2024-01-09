use core::ffi::CStr;

use crate::ffi::OsString;
use crate::fmt;
use crate::hash::{Hash, Hasher};
use crate::io::{self, BorrowedCursor, IoSlice, IoSliceMut, SeekFrom};
use crate::os::amjad_os::io::{AsFd, AsRawFd, BorrowedFd, FromRawFd, IntoRawFd, RawFd};
use crate::path::{Path, PathBuf};
use crate::sys::common::small_c_string::run_path_with_cstr;
use crate::sys::time::SystemTime;
use crate::sys_common::{AsInner, AsInnerMut, FromInner, IntoInner};

use super::fd::FileDesc;
use super::syscall_to_io_error;

pub struct File(FileDesc);

pub struct FileAttr(!);

pub struct ReadDir(!);

pub struct DirEntry(!);

#[derive(Clone, Debug)]
pub struct OpenOptions {}

#[derive(Copy, Clone, Debug, Default)]
pub struct FileTimes {}

pub struct FilePermissions(!);

pub struct FileType(!);

#[derive(Debug)]
pub struct DirBuilder {}

impl FileAttr {
    pub fn size(&self) -> u64 {
        todo!()
    }

    pub fn perm(&self) -> FilePermissions {
        todo!()
    }

    pub fn file_type(&self) -> FileType {
        todo!()
    }

    pub fn modified(&self) -> io::Result<SystemTime> {
        todo!()
    }

    pub fn accessed(&self) -> io::Result<SystemTime> {
        todo!()
    }

    pub fn created(&self) -> io::Result<SystemTime> {
        todo!()
    }
}

impl Clone for FileAttr {
    fn clone(&self) -> FileAttr {
        self.0
    }
}

impl FilePermissions {
    pub fn readonly(&self) -> bool {
        self.0
    }

    pub fn set_readonly(&mut self, _readonly: bool) {
        self.0
    }
}

impl Clone for FilePermissions {
    fn clone(&self) -> FilePermissions {
        self.0
    }
}

impl PartialEq for FilePermissions {
    fn eq(&self, _other: &FilePermissions) -> bool {
        self.0
    }
}

impl Eq for FilePermissions {}

impl fmt::Debug for FilePermissions {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0
    }
}

impl FileTimes {
    pub fn set_accessed(&mut self, _t: SystemTime) {}
    pub fn set_modified(&mut self, _t: SystemTime) {}
}

impl FileType {
    pub fn is_dir(&self) -> bool {
        self.0
    }

    pub fn is_file(&self) -> bool {
        self.0
    }

    pub fn is_symlink(&self) -> bool {
        self.0
    }
}

impl Clone for FileType {
    fn clone(&self) -> FileType {
        self.0
    }
}

impl Copy for FileType {}

impl PartialEq for FileType {
    fn eq(&self, _other: &FileType) -> bool {
        self.0
    }
}

impl Eq for FileType {}

impl Hash for FileType {
    fn hash<H: Hasher>(&self, _h: &mut H) {
        self.0
    }
}

impl fmt::Debug for FileType {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0
    }
}

impl fmt::Debug for ReadDir {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0
    }
}

impl Iterator for ReadDir {
    type Item = io::Result<DirEntry>;

    fn next(&mut self) -> Option<io::Result<DirEntry>> {
        self.0
    }
}

impl DirEntry {
    pub fn path(&self) -> PathBuf {
        self.0
    }

    pub fn file_name(&self) -> OsString {
        self.0
    }

    pub fn metadata(&self) -> io::Result<FileAttr> {
        self.0
    }

    pub fn file_type(&self) -> io::Result<FileType> {
        self.0
    }
}

impl OpenOptions {
    pub fn new() -> OpenOptions {
        OpenOptions {}
    }

    pub fn read(&mut self, _read: bool) {}
    pub fn write(&mut self, _write: bool) {}
    pub fn append(&mut self, _append: bool) {}
    pub fn truncate(&mut self, _truncate: bool) {}
    pub fn create(&mut self, _create: bool) {}
    pub fn create_new(&mut self, _create_new: bool) {}
}

impl File {
    pub fn open(path: &Path, opts: &OpenOptions) -> io::Result<File> {
        run_path_with_cstr(path, |path| Self::openc(path, opts))
    }

    pub fn openc(path: &CStr, _opts: &OpenOptions) -> io::Result<File> {
        let flags = 0;
        let access_mode = 0;

        let fd = unsafe {
            user_std::io::syscall_open(path, access_mode, flags).map_err(syscall_to_io_error)
        }?;

        Ok(File(unsafe { FileDesc::from_raw_fd(fd as usize) }))
    }

    pub fn into_inner(self) -> FileDesc {
        self.0
    }

    pub fn file_attr(&self) -> io::Result<FileAttr> {
        todo!()
    }

    pub fn fsync(&self) -> io::Result<()> {
        todo!()
    }

    pub fn datasync(&self) -> io::Result<()> {
        todo!()
    }

    pub fn truncate(&self, _size: u64) -> io::Result<()> {
        todo!()
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }

    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.0.read_vectored(bufs)
    }

    pub fn is_read_vectored(&self) -> bool {
        self.0.is_read_vectored()
    }

    pub fn read_buf(&self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        self.0.read_buf(cursor)
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

    pub fn flush(&self) -> io::Result<()> {
        todo!()
    }

    pub fn seek(&self, _pos: SeekFrom) -> io::Result<u64> {
        todo!()
    }

    pub fn duplicate(&self) -> io::Result<File> {
        self.0.duplicate().map(File)
    }

    pub fn set_permissions(&self, _perm: FilePermissions) -> io::Result<()> {
        todo!()
    }

    pub fn set_times(&self, _times: FileTimes) -> io::Result<()> {
        todo!()
    }
}

impl AsInner<FileDesc> for File {
    #[inline]
    fn as_inner(&self) -> &FileDesc {
        &self.0
    }
}

impl AsInnerMut<FileDesc> for File {
    #[inline]
    fn as_inner_mut(&mut self) -> &mut FileDesc {
        &mut self.0
    }
}

impl IntoInner<FileDesc> for File {
    fn into_inner(self) -> FileDesc {
        self.0
    }
}

impl FromInner<FileDesc> for File {
    fn from_inner(file_desc: FileDesc) -> Self {
        Self(file_desc)
    }
}

impl AsFd for File {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl AsRawFd for File {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl IntoRawFd for File {
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

impl FromRawFd for File {
    unsafe fn from_raw_fd(raw_fd: RawFd) -> Self {
        Self(unsafe { FromRawFd::from_raw_fd(raw_fd) })
    }
}

impl DirBuilder {
    pub fn new() -> DirBuilder {
        DirBuilder {}
    }

    pub fn mkdir(&self, _p: &Path) -> io::Result<()> {
        todo!("mkdir")
    }
}

impl fmt::Debug for File {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

pub fn readdir(_p: &Path) -> io::Result<ReadDir> {
    todo!("readdir")
}

pub fn unlink(_p: &Path) -> io::Result<()> {
    todo!("unlink")
}

pub fn rename(_old: &Path, _new: &Path) -> io::Result<()> {
    todo!("rename")
}

pub fn set_perm(_p: &Path, perm: FilePermissions) -> io::Result<()> {
    match perm.0 {}
}

pub fn rmdir(_p: &Path) -> io::Result<()> {
    todo!("rmdir")
}

pub fn remove_dir_all(_path: &Path) -> io::Result<()> {
    todo!("remove_dir_all")
}

pub fn try_exists(_path: &Path) -> io::Result<bool> {
    todo!("try_exists")
}

pub fn readlink(_p: &Path) -> io::Result<PathBuf> {
    todo!("readlink")
}

pub fn symlink(_original: &Path, _link: &Path) -> io::Result<()> {
    todo!("symlink")
}

pub fn link(_src: &Path, _dst: &Path) -> io::Result<()> {
    todo!("link")
}

pub fn stat(_p: &Path) -> io::Result<FileAttr> {
    todo!("stat")
}

pub fn lstat(_p: &Path) -> io::Result<FileAttr> {
    todo!("lstat")
}

pub fn canonicalize(_p: &Path) -> io::Result<PathBuf> {
    todo!("canonicalize")
}

pub fn copy(_from: &Path, _to: &Path) -> io::Result<u64> {
    todo!("copy")
}
