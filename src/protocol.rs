use derive_try_from_primitive::TryFromPrimitive;
use os_pipe::{PipeReader, PipeWriter};
use std::{
    io,
    io::{Read, Write},
    process::Child,
};
use thiserror::Error;

use crate::serialization::SerializationEndpoint;

#[repr(u8)]
#[derive(TryFromPrimitive, Copy, Clone, Debug, PartialEq, Eq)]
pub enum ProtocolConstant {
    Hello = 0x01,

    Goodbye = 0xff,
}

impl ProtocolConstant {
    pub fn expect(self, expected: ProtocolConstant) -> Result<(), ProtocolError>{
        if self == expected {
            Ok(())
        }else{
            Err(ProtocolError::UnexpectedProtocolConstant{got: self, expected})
        }
    }
}

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Unknown protocol constant {0}")]
    UnknownProtocolConstant(u8),

    #[error("Unexpected protocol constant. Expected {expected:?}, but Got {got:?}")]
    UnexpectedProtocolConstant{got: ProtocolConstant, expected: ProtocolConstant},
}

impl From<ProtocolError> for io::Error {
    fn from(e: ProtocolError) -> Self {
        io::Error::new(io::ErrorKind::Other, e)
    }
}

pub struct ControllerProcess {
    pub(crate) subordinate: Child,
    pub(crate) cmd_out: PipeWriter,
    pub(crate) cmd_in: PipeReader,
}

pub struct SubordinateProcess {
    pub(crate) cmd_out: PipeWriter,
    pub(crate) cmd_in: PipeReader,
}

pub struct Pipes<'a> {
    pub reader: &'a mut PipeReader,
    pub writer: &'a mut PipeWriter,
}

pub trait Endpoint {
    fn pipes(&mut self) -> Pipes;

    fn hello(&mut self) -> io::Result<()>;

    fn goodbye(&mut self) -> io::Result<()>;
}

impl Endpoint for ControllerProcess {
    fn pipes(&mut self) -> Pipes {
        Pipes {
            reader: &mut self.cmd_in,
            writer: &mut self.cmd_out,
        }
    }

    fn hello(&mut self) -> io::Result<()> {
        let result = self
        .write_protocol(ProtocolConstant::Hello)?
        .read_protocol()?;
        assert!(result == ProtocolConstant::Hello);

        Ok(())
    }

    fn goodbye(&mut self) -> io::Result<()> {
        todo!()
    }
}

impl ControllerProcess {
    pub fn shutdown(mut self) -> io::Result<()> {
        self.cmd_out
            .write_all(std::slice::from_ref(&(ProtocolConstant::Goodbye as u8)))?;
        self.subordinate.wait()?;
        Ok(())
    }
}

impl Endpoint for SubordinateProcess {
    fn pipes(&mut self) -> Pipes {
        Pipes {
            reader: &mut self.cmd_in,
            writer: &mut self.cmd_out,
        }
    }

    fn hello(&mut self) -> io::Result<()> {
        let pipes = self.pipes();
        let mut buf = [0];
        pipes.reader.read_exact(&mut buf)?;
        assert!(buf[0] == ProtocolConstant::Hello as u8);
        // subordinate says hello second
        pipes
            .writer
            .write_all(std::slice::from_ref(&(ProtocolConstant::Hello as u8)))?;
        Ok(())
    }

    fn goodbye(&mut self) -> io::Result<()> {
        todo!()
    }
}
