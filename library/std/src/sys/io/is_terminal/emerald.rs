use emerald_std::io::FileMeta;

use crate::os::emerald::io::{AsFd, AsRawFd};

pub fn is_terminal(file: &impl AsFd) -> bool {
    let mut meta = FileMeta::IsTerminal(false);
    unsafe {
        emerald_std::io::syscall_get_file_meta(file.as_fd().as_raw_fd(), &mut meta)
            .expect("syscall_get_file_meta failed");
    }

    match meta {
        FileMeta::IsTerminal(is_terminal) => is_terminal,
        _ => unreachable!(),
    }
}
