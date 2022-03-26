use derive_try_from_primitive::TryFromPrimitive;
use std::io;
use thiserror::Error;

use crate::{
    protocol::*,
    serialization::SerializedType,
    GenericValueBoxed,
};

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Unknown protocol constant {0}")]
    UnknownProtocolConstant(u8),

    #[error("Unknown error type {0}")]
    UnknownErrorType(u8),

    #[error("Unknown log type {0}")]
    UnknownLogType(u8),

    #[error("Unexpected protocol constant. Expected {expected:?}, but Got {got:?}")]
    UnexpectedProtocolConstant {
        got: ProtocolConstant,
        expected: ProtocolConstant,
    },

    #[error("Inappropriate protocol constant {0:?}")]
    InappropriateProtocolConstant(ProtocolConstant),

    #[error("Unknown generic type {0}")]
    UnknownGenericType(u8),
}

impl From<ProtocolError> for io::Error {
    fn from(e: ProtocolError) -> Self {
        io::Error::new(io::ErrorKind::Other, e)
    }
}

#[derive(Error, Debug)]
#[error("Unexpected generic type. Expected {expected_type:?}, Got {actual_type:?}")]
pub struct UnexpectedGenericType {
    pub expected_type: SerializedType,
    pub actual_type: SerializedType,
}

impl From<UnexpectedGenericType> for io::Error {
    fn from(e: UnexpectedGenericType) -> Self {
        io::Error::new(io::ErrorKind::Other, e)
    }
}

#[derive(Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum RemoteErrorType {
    UserspaceException,
    ProtocolError,
    GenericTypeError,
}

#[derive(Error, Debug)]
#[error("Remote Error {err_type:?}")]
pub struct RemoteError {
    pub logs: Vec<LogEntry>,
    pub err_type: RemoteErrorType,
    pub aux: Vec<GenericValueBoxed>,
}

impl From<RemoteError> for io::Error {
    fn from(e: RemoteError) -> Self {
        io::Error::new(io::ErrorKind::Other, e)
    }
}
