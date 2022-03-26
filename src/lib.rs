// use libc::{dup2, STDERR_FILENO, STDOUT_FILENO};
use os_pipe::{PipeReader, PipeWriter};
use std::{
    io::Result,
    os::unix::prelude::{AsRawFd, FromRawFd},
    process::{Command, Stdio},
    str::FromStr,
};

mod pipe;
use pipe::*;

mod err;
pub use err::*;

mod endpoint;
pub use endpoint::{sealed::*, *};

mod protocol;
pub use protocol::*;

mod serialization;
pub use serialization::*;

pub trait StartSubordinateProcess {
    fn start_subordinate_process(&mut self) -> Result<ControllerProcess>;
}

const SUB_IN_ENV: &str = "UFO_SUBORDINATE_PIPEFD_IN";
const SUB_OUT_ENV: &str = "UFO_SUBORDINATE_PIPEFD_OUT";

impl StartSubordinateProcess for Command {
    fn start_subordinate_process(&mut self) -> Result<ControllerProcess> {
        let parent_to_child = pipe2_nocloexec()?; // packetPipe2()?;
        let child_to_parent = pipe2_nocloexec()?;

        let child_pipefd_in = parent_to_child.0;
        let child_pipefd_out = child_to_parent.1;

        let subordinate = self
            .stderr(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stdin(Stdio::null())
            .env(SUB_IN_ENV, child_pipefd_in.as_raw_fd().to_string())
            .env(SUB_OUT_ENV, child_pipefd_out.as_raw_fd().to_string())
            .spawn()?;

        let mut controller =
            ControllerProcess::new(subordinate, parent_to_child.1, child_to_parent.0);

        controller.hello()?;

        // child process good and started, drop our copies of their sides of the pipes
        std::mem::drop(child_pipefd_in);
        std::mem::drop(child_pipefd_out);

        Ok(controller)
    }
}

pub fn subordinate_begin() -> Result<SubordinateProcess> {
    let pipe_in =
        std::env::var(SUB_IN_ENV).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let pipe_out = std::env::var(SUB_OUT_ENV)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let pipe_in =
        i32::from_str(&pipe_in).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let pipe_out =
        i32::from_str(&pipe_out).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let cmd_in = unsafe { PipeReader::from_raw_fd(pipe_in) };
    let cmd_out = unsafe { PipeWriter::from_raw_fd(pipe_out) };

    // let (stdout_r, stdout_w) = pipe2_nocloexec()?;
    // let (stderr_r, stderr_w) = pipe2_nocloexec()?;

    // unsafe{
    //     dup2(stdout_w.as_raw_fd(), STDOUT_FILENO);
    //     dup2(stderr_w.as_raw_fd(), STDERR_FILENO);
    // }

    let mut sub = SubordinateProcess { cmd_in, cmd_out };

    sub.hello()?;

    Ok(sub)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
