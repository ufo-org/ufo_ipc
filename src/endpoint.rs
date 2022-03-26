use os_pipe::{PipeReader, PipeWriter};
// use std::{process::Child, thread::Thread};
use std::{process::Child};
pub struct ControllerProcess {
    pub(crate) subordinate: Child,
    pub(crate) cmd_out: PipeWriter,
    pub(crate) cmd_in: PipeReader,

    pub(crate) id_ctr: u64,
}

impl ControllerProcess {
    pub(crate) fn new(subordinate: Child, cmd_out: PipeWriter, cmd_in: PipeReader) -> Self {
        ControllerProcess {
            subordinate,
            cmd_in,
            cmd_out,
            id_ctr: 0,
        }
    }
}

// pub(crate) struct ConsoleReaderThread {
//     pub thread: Thread,
//     pub reader: PipeReader,
//     pub writer: PipeWriter,
// }

pub struct SubordinateProcess {
    pub(crate) cmd_out: PipeWriter,
    pub(crate) cmd_in: PipeReader,
    // pub(crate) stdout_reader: ConsoleReaderThread,
    // pub(crate) stderr_reader: ConsoleReaderThread,
}

pub(crate) mod sealed {
    use os_pipe::{PipeReader, PipeWriter};
    use std::{io, io::Write};

    use crate::*;

    pub struct Pipes<'a> {
        pub reader: &'a mut PipeReader,
        pub writer: &'a mut PipeWriter,
    }

    pub trait Endpoint {
        fn pipes(&mut self) -> Pipes;

        fn flush(&mut self) -> io::Result<&mut Self> {
            self.pipes().writer.flush()?;
            Ok(self)
        }
    }

    impl Endpoint for ControllerProcess {
        fn pipes(&mut self) -> Pipes {
            Pipes {
                reader: &mut self.cmd_in,
                writer: &mut self.cmd_out,
            }
        }
    }

    impl Endpoint for SubordinateProcess {
        fn pipes(&mut self) -> Pipes {
            Pipes {
                reader: &mut self.cmd_in,
                writer: &mut self.cmd_out,
            }
        }
    }
}
