use nix::{fcntl::OFlag, Result};
use os_pipe::{PipeReader, PipeWriter};
use std::os::unix::prelude::FromRawFd;

pub fn pipe2_nocloexec() -> Result<(PipeReader, PipeWriter)> {
    nix::unistd::pipe2(OFlag::empty()) // O_DIRECT results in a "packet pipe" where reads cut at write boundaries
        .map(|(r, w)| unsafe { (PipeReader::from_raw_fd(r), PipeWriter::from_raw_fd(w)) })
}
