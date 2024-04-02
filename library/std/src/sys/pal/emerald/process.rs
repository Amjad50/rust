use core::ffi::c_char;
use core::ffi::CStr;
use core::ptr;

use alloc_crate::ffi::CString;
use emerald_std::io::FD_STDERR;
use emerald_std::io::FD_STDIN;
use emerald_std::io::FD_STDOUT;
use emerald_std::process::SpawnFileMapping;
use emerald_std::SyscallError;

use crate::ffi::OsStr;
use crate::fmt;
use crate::io;
use crate::num::NonZeroI32;
use crate::os::emerald::io::AsRawFd;
use crate::os::emerald::io::IntoRawFd;
use crate::os::emerald::io::RawFd;
use crate::path::Path;
use crate::sys::fs::File;
use crate::sys::pal::emerald::syscall_to_io_error;
use crate::sys::pipe::AnonPipe;
use crate::sys_common::process::{CommandEnv, CommandEnvs};

pub use crate::ffi::OsString as EnvKey;

use super::fd::FileDesc;
use super::pipe;

struct Argv(Vec<*const c_char>);

////////////////////////////////////////////////////////////////////////////////
// Command
////////////////////////////////////////////////////////////////////////////////

pub struct Command {
    program: CString,
    args: Vec<CString>,
    argv: Argv,
    env: CommandEnv,

    stdin: Option<Stdio>,
    stdout: Option<Stdio>,
    stderr: Option<Stdio>,

    spawned: bool,
}

// passed back to std::process with the pipes connected to the child, if any
// were requested
pub struct StdioPipes {
    pub stdin: Option<AnonPipe>,
    pub stdout: Option<AnonPipe>,
    pub stderr: Option<AnonPipe>,
}

pub enum Stdio {
    Inherit,
    Null,
    MakePipe,
    Fd(FileDesc),
}

// used to configure file mappings for the child
pub struct ChildPipes {
    pub stdin: ChildStdio,
    pub stdout: ChildStdio,
    pub stderr: ChildStdio,
}

pub enum ChildStdio {
    Inherit,
    Owned(FileDesc),
    Explicit(RawFd),
}

impl ChildStdio {
    pub fn into_file_mappings(self) -> Option<SpawnFileMapping> {
        match self {
            ChildStdio::Inherit => None,
            ChildStdio::Owned(fd) => Some(SpawnFileMapping { src_fd: fd.into_raw_fd(), dst_fd: 0 }),
            ChildStdio::Explicit(fd) => Some(SpawnFileMapping { src_fd: fd, dst_fd: 0 }),
        }
    }
}

impl Command {
    pub fn new(program: &OsStr) -> Command {
        let program = CString::new(program.as_encoded_bytes()).unwrap();
        Command {
            program: program.clone(),
            argv: Argv(vec![program.as_ptr(), ptr::null()]),
            args: vec![program],
            env: Default::default(),
            stdin: None,
            stdout: None,
            stderr: None,

            spawned: false,
        }
    }

    pub fn arg(&mut self, arg: &OsStr) {
        // Overwrite the trailing null pointer in `argv` and then add a new null
        // pointer.
        let arg = CString::new(arg.as_encoded_bytes()).unwrap();
        self.argv.0[self.args.len()] = arg.as_ptr();
        self.argv.0.push(ptr::null());

        // Also make sure we keep track of the owned value to schedule a
        // destructor for this memory.
        self.args.push(arg);
    }

    pub fn env_mut(&mut self) -> &mut CommandEnv {
        &mut self.env
    }

    pub fn cwd(&mut self, _dir: &OsStr) {}

    pub fn stdin(&mut self, stdin: Stdio) {
        self.stdin = Some(stdin);
    }

    pub fn stdout(&mut self, stdout: Stdio) {
        self.stdout = Some(stdout);
    }

    pub fn stderr(&mut self, stderr: Stdio) {
        self.stderr = Some(stderr);
    }

    pub fn get_program(&self) -> &OsStr {
        // Safety: we have used `as_encoded_bytes` to create this `CString`, so this is valid
        unsafe { OsStr::from_encoded_bytes_unchecked(self.program.as_bytes()) }
    }

    pub fn get_program_cstr(&self) -> &CStr {
        &self.program
    }

    pub fn get_args(&self) -> CommandArgs<'_> {
        let mut iter = self.args.iter();
        iter.next();
        CommandArgs { iter }
    }

    pub fn get_envs(&self) -> CommandEnvs<'_> {
        self.env.iter()
    }

    fn get_argv(&self) -> &Vec<*const c_char> {
        &self.argv.0
    }

    pub fn get_current_dir(&self) -> Option<&Path> {
        None
    }

    fn setup_io(
        &mut self,
        mut default: Stdio,
        needs_stdin: bool,
    ) -> io::Result<(StdioPipes, ChildPipes)> {
        let mut null = Stdio::Null;
        let default_stdin = if needs_stdin { &mut default } else { &mut null };
        let stdin = self.stdin.as_mut().unwrap_or(default_stdin);
        let (their_stdin, our_stdin) = stdin.to_child_stdio(true)?;

        let stdout = self.stdout.as_mut().unwrap_or(&mut default);
        let (their_stdout, our_stdout) = stdout.to_child_stdio(false)?;

        let stderr = self.stderr.as_mut().unwrap_or(&mut default);
        let (their_stderr, our_stderr) = stderr.to_child_stdio(false)?;
        let ours = StdioPipes { stdin: our_stdin, stdout: our_stdout, stderr: our_stderr };
        let theirs = ChildPipes { stdin: their_stdin, stdout: their_stdout, stderr: their_stderr };
        Ok((ours, theirs))
    }

    pub fn spawn(
        &mut self,
        default: Stdio,
        needs_stdin: bool,
    ) -> io::Result<(Process, StdioPipes)> {
        // TODO: add support for multiple spawns.
        //       this requires `dups` or `clone` for files.
        //       the reason its like that is that `Stdio::Fd` must be moved
        //       so we need a way to clone the file
        if self.spawned {
            return Err(io::Error::new(io::ErrorKind::Other, "Command can only be spawned once"));
        }

        let (ours, theirs) = self.setup_io(default, needs_stdin)?;
        self.spawned = true;

        // setup 3 mappings as the max, and only use what's needed
        let mut file_mappings = [SpawnFileMapping { src_fd: 0, dst_fd: 0 }; 3];
        let mut mappings_i = 0;

        if let Some(mut file_map) = theirs.stdin.into_file_mappings() {
            file_map.dst_fd = FD_STDIN;
            file_mappings[mappings_i] = file_map;
            mappings_i += 1;
        }
        if let Some(mut file_map) = theirs.stdout.into_file_mappings() {
            file_map.dst_fd = FD_STDOUT;
            file_mappings[mappings_i] = file_map;
            mappings_i += 1;
        }
        if let Some(mut file_map) = theirs.stderr.into_file_mappings() {
            file_map.dst_fd = FD_STDERR;
            file_mappings[mappings_i] = file_map;
            mappings_i += 1;
        }

        let pid = unsafe {
            emerald_std::process::spawn(
                self.get_program_cstr(),
                self.get_argv(),
                &file_mappings[..mappings_i],
            )
            .map_err(syscall_to_io_error)?
        };
        Ok((Process { pid: pid as u32 }, ours))
    }

    pub fn output(&mut self) -> io::Result<(ExitStatus, Vec<u8>, Vec<u8>)> {
        let (proc, pipes) = self.spawn(Stdio::MakePipe, false)?;
        crate::sys_common::process::wait_with_output(proc, pipes)
    }
}

impl Stdio {
    pub fn to_child_stdio(&mut self, readable: bool) -> io::Result<(ChildStdio, Option<AnonPipe>)> {
        match self {
            Stdio::Inherit => Ok((ChildStdio::Inherit, None)),

            // Make sure that the source descriptors are not an stdio
            // descriptor, otherwise the order which we set the child's
            // descriptors may blow away a descriptor which we are hoping to
            // save. For example, suppose we want the child's stderr to be the
            // parent's stdout, and the child's stdout to be the parent's
            // stderr. No matter which we dup first, the second will get
            // overwritten prematurely.
            Stdio::Fd(_) => {
                // replace self with inherit
                // TODO: this is not good, replace with `dups` or cloning the file somehow
                let fd = core::mem::replace(self, Stdio::Inherit);

                let fd = match fd {
                    Stdio::Fd(fd) => fd,
                    _ => unreachable!(),
                };

                if fd.as_raw_fd() <= FD_STDERR {
                    // TODO: add support for passing stdio fds
                    Err(io::Error::new(
                        io::ErrorKind::Other,
                        "stdio fds are not supported to forward to child process, use inherit instead or makepipe",
                    ))
                } else {
                    // move the fd
                    Ok((ChildStdio::Explicit(fd.into_raw_fd()), None))
                }
            }

            Stdio::MakePipe => {
                let (reader, writer) = pipe::anon_pipe()?;
                let (ours, theirs) = if readable { (writer, reader) } else { (reader, writer) };
                Ok((ChildStdio::Owned(theirs.into_inner()), Some(ours)))
            }

            // TODO: replace with null device
            Stdio::Null => Ok((ChildStdio::Inherit, None)),
        }
    }
}

impl From<AnonPipe> for Stdio {
    fn from(pipe: AnonPipe) -> Stdio {
        Stdio::Fd(pipe.into_inner())
    }
}

impl From<io::Stdout> for Stdio {
    fn from(_: io::Stdout) -> Stdio {
        // FIXME: This is wrong.
        // Instead, the Stdio we have here should be a unit struct.
        panic!("unsupported")
    }
}

impl From<io::Stderr> for Stdio {
    fn from(_: io::Stderr) -> Stdio {
        // FIXME: This is wrong.
        // Instead, the Stdio we have here should be a unit struct.
        panic!("unsupported")
    }
}

impl From<File> for Stdio {
    fn from(file: File) -> Stdio {
        Stdio::Fd(file.into_inner())
    }
}

impl fmt::Debug for Command {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
#[non_exhaustive]
pub struct ExitStatus(i32);

impl ExitStatus {
    pub fn exit_ok(&self) -> Result<(), ExitStatusError> {
        match self.0 {
            0 => Ok(()),
            _ => Err(ExitStatusError(self.0)),
        }
    }

    pub fn code(&self) -> Option<i32> {
        Some(self.0)
    }

    pub fn into_raw(&self) -> i32 {
        self.0
    }
}

/// Converts a raw `i32` to a type-safe `ExitStatus` by wrapping it without copying.
impl From<i32> for ExitStatus {
    fn from(a: i32) -> ExitStatus {
        ExitStatus(a)
    }
}

impl fmt::Display for ExitStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "code: {}", self.0)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct ExitStatusError(i32);

impl Into<ExitStatus> for ExitStatusError {
    fn into(self) -> ExitStatus {
        ExitStatus(self.0)
    }
}

impl ExitStatusError {
    pub fn code(self) -> Option<NonZeroI32> {
        assert_ne!(self.0, 0);
        Some(unsafe { NonZeroI32::new_unchecked(self.0) })
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ExitCode {
    Success,
    Failure,
}

impl ExitCode {
    pub const SUCCESS: ExitCode = ExitCode::Success;
    pub const FAILURE: ExitCode = ExitCode::Failure;

    pub fn as_i32(&self) -> i32 {
        match *self {
            ExitCode::Success => 0,
            ExitCode::Failure => 1,
        }
    }
}

impl From<u8> for ExitCode {
    fn from(code: u8) -> Self {
        match code {
            0 => Self::Success,
            1..=255 => Self::Failure,
        }
    }
}

pub struct Process {
    pid: u32,
}

impl Process {
    pub fn id(&self) -> u32 {
        self.pid
    }

    pub fn kill(&mut self) -> io::Result<()> {
        todo!()
    }

    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        let status_code = unsafe {
            emerald_std::process::wait_for_pid(self.pid as u64, true)
                .map_err(syscall_to_io_error)?
        };
        Ok(ExitStatus(status_code as i32))
    }

    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        let status_code = unsafe { emerald_std::process::wait_for_pid(self.pid as u64, false) };
        match status_code {
            Ok(status_code) => Ok(Some(ExitStatus(status_code as i32))),
            Err(SyscallError::ProcessStillRunning) => Ok(None),
            Err(e) => Err(syscall_to_io_error(e)),
        }
    }
}

pub struct CommandArgs<'a> {
    iter: crate::slice::Iter<'a, CString>,
}

impl<'a> Iterator for CommandArgs<'a> {
    type Item = &'a OsStr;
    fn next(&mut self) -> Option<&'a OsStr> {
        // Safety: these args were created with `as_encoded_bytes`
        self.iter.next().map(|cs| unsafe { OsStr::from_encoded_bytes_unchecked(cs.as_bytes()) })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> ExactSizeIterator for CommandArgs<'a> {
    fn len(&self) -> usize {
        self.iter.len()
    }
    fn is_empty(&self) -> bool {
        self.iter.is_empty()
    }
}

impl<'a> fmt::Debug for CommandArgs<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter.clone()).finish()
    }
}
