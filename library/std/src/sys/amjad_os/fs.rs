use core::ffi::CStr;

use user_std::io::FileStat;

use crate::ffi::OsString;
use crate::fmt;
use crate::hash::Hash;
use crate::io::{self, BorrowedCursor, IoSlice, IoSliceMut, SeekFrom};
use crate::os::amjad_os::io::{AsFd, AsRawFd, BorrowedFd, FromRawFd, IntoRawFd, OwnedFd, RawFd};
use crate::os::amjad_os::prelude::OsStringExt;
use crate::path::{Path, PathBuf};
use crate::sys::common::small_c_string::run_path_with_cstr;
use crate::sys::time::SystemTime;
use crate::sys_common::{AsInner, AsInnerMut, FromInner, IntoInner};

use super::fd::FileDesc;
use super::syscall_to_io_error;

pub struct File {
    path: PathBuf,
    fd: FileDesc,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FileAttr(FileStat);

pub struct ReadDir {
    path: PathBuf,
    fd: OwnedFd,
    fetched_entries: Vec<DirEntry>,
    finished: bool,
}

pub struct DirEntry {
    system_entry: user_std::io::DirEntry,
    parent_path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct OpenOptions {}

#[derive(Copy, Clone, Debug, Default)]
pub struct FileTimes {}

pub struct FilePermissions(!);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FileType(user_std::io::FileType);

#[derive(Debug)]
pub struct DirBuilder {}

impl FileAttr {
    pub fn size(&self) -> u64 {
        self.0.size
    }

    pub fn perm(&self) -> FilePermissions {
        todo!()
    }

    pub fn file_type(&self) -> FileType {
        FileType(self.0.file_type)
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
        self.0 == user_std::io::FileType::Directory
    }

    pub fn is_file(&self) -> bool {
        self.0 == user_std::io::FileType::File
    }

    pub fn is_symlink(&self) -> bool {
        false
    }
}

impl ReadDir {
    fn new(path: &Path) -> io::Result<ReadDir> {
        let raw_fd = run_path_with_cstr(path, |path| unsafe {
            user_std::io::syscall_open_dir(path).map_err(syscall_to_io_error)
        })?;

        Ok(ReadDir {
            path: path.to_owned(),
            fd: unsafe { FromRawFd::from_raw_fd(raw_fd) },
            fetched_entries: Vec::new(),
            finished: false,
        })
    }

    fn populate_next_entries(&mut self) -> io::Result<bool> {
        assert!(self.fetched_entries.is_empty());

        let mut entries = [user_std::io::DirEntry::default(); 16];
        let num_entries = unsafe {
            user_std::io::syscall_read_dir(self.fd.as_raw_fd(), &mut entries)
                .map_err(syscall_to_io_error)?
        };

        if num_entries == 0 {
            self.finished = true;
            return Ok(false);
        }

        // NOTE: this is annoying me, I don't want to `copy` since I know that the value is `taken` here and never used again
        // would be good to find a better way
        for entry in &entries[..num_entries] {
            self.fetched_entries
                .push(DirEntry { system_entry: *entry, parent_path: self.path.clone() });
        }

        Ok(true)
    }

    // This is safe, it just panics if there are no entries, which should never happen
    // caller must ensure that there are entries
    fn pop_next_unchecked(&mut self) -> DirEntry {
        self.fetched_entries.pop().unwrap()
    }
}

impl fmt::Debug for ReadDir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.path, f)
    }
}

impl Iterator for ReadDir {
    type Item = io::Result<DirEntry>;

    fn next(&mut self) -> Option<io::Result<DirEntry>> {
        if self.finished {
            return None;
        }

        if self.fetched_entries.is_empty() {
            match self.populate_next_entries() {
                Ok(true) => {}                 // got more data
                Ok(false) => return None,      // finished
                Err(e) => return Some(Err(e)), // error
            }
        }

        let entry = self.pop_next_unchecked();
        Some(Ok(entry))
    }
}

impl DirEntry {
    pub fn path(&self) -> PathBuf {
        self.parent_path.join(self.file_name())
    }

    pub fn file_name(&self) -> OsString {
        OsString::from_vec(self.system_entry.filename_cstr().to_bytes().to_vec())
    }

    pub fn metadata(&self) -> io::Result<FileAttr> {
        Ok(FileAttr(self.system_entry.stat))
    }

    pub fn file_type(&self) -> io::Result<FileType> {
        Ok(FileType(self.system_entry.stat.file_type))
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
        let fd = run_path_with_cstr(path, |path| Self::openc(path, opts))?;

        Ok(File { path: path.to_owned(), fd })
    }

    fn openc(path: &CStr, _opts: &OpenOptions) -> io::Result<FileDesc> {
        let flags = 0;
        let access_mode = 0;

        let fd = unsafe {
            user_std::io::syscall_open(path, access_mode, flags).map_err(syscall_to_io_error)
        }?;

        Ok(unsafe { FileDesc::from_raw_fd(fd as usize) })
    }

    pub fn into_inner(self) -> FileDesc {
        self.fd
    }

    pub fn file_attr(&self) -> io::Result<FileAttr> {
        // TODO: optimize by having a syscall for `fd`
        stat(&self.path)
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
        self.fd.read(buf)
    }

    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.fd.read_vectored(bufs)
    }

    pub fn is_read_vectored(&self) -> bool {
        self.fd.is_read_vectored()
    }

    pub fn read_buf(&self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        self.fd.read_buf(cursor)
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        self.fd.write(buf)
    }

    pub fn write_vectored(&self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        self.fd.write_vectored(bufs)
    }

    pub fn is_write_vectored(&self) -> bool {
        self.fd.is_write_vectored()
    }

    pub fn flush(&self) -> io::Result<()> {
        todo!()
    }

    pub fn seek(&self, _pos: SeekFrom) -> io::Result<u64> {
        todo!()
    }

    pub fn duplicate(&self) -> io::Result<File> {
        let fd = self.fd.duplicate()?;
        Ok(File { path: self.path.clone(), fd })
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
        &self.fd
    }
}

impl AsInnerMut<FileDesc> for File {
    #[inline]
    fn as_inner_mut(&mut self) -> &mut FileDesc {
        &mut self.fd
    }
}

impl IntoInner<FileDesc> for File {
    fn into_inner(self) -> FileDesc {
        self.fd
    }
}

impl FromInner<FileDesc> for File {
    fn from_inner(file_desc: FileDesc) -> Self {
        Self { path: PathBuf::new(), fd: file_desc }
    }
}

impl AsFd for File {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

impl AsRawFd for File {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
}

impl IntoRawFd for File {
    fn into_raw_fd(self) -> RawFd {
        self.fd.into_raw_fd()
    }
}

impl FromRawFd for File {
    unsafe fn from_raw_fd(raw_fd: RawFd) -> Self {
        File { path: PathBuf::new(), fd: unsafe { FromRawFd::from_raw_fd(raw_fd) } }
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.path, f)
    }
}

pub fn readdir(p: &Path) -> io::Result<ReadDir> {
    ReadDir::new(p)
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

pub fn stat(p: &Path) -> io::Result<FileAttr> {
    run_path_with_cstr(p, |c_path| {
        // will be overwritten by syscall
        let mut stat = FileStat::default();

        unsafe {
            user_std::io::syscall_stat(c_path, &mut stat)
                .map_err(syscall_to_io_error)
                .map(|_| FileAttr(stat))
        }
    })
}

pub fn lstat(p: &Path) -> io::Result<FileAttr> {
    // TODO: add impl to symlink or similar things
    stat(p)
}

pub fn canonicalize(_p: &Path) -> io::Result<PathBuf> {
    todo!("canonicalize")
}

pub fn copy(_from: &Path, _to: &Path) -> io::Result<u64> {
    todo!("copy")
}
