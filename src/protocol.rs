use crate::serialization::{sealed::SerializationEndpoint};
use crate::serialization::{GenericValueBoxed, GenericValueRef};
use crate::*;
use derive_try_from_primitive::TryFromPrimitive;
use std::{io, result::Result};

#[repr(u8)]
#[derive(TryFromPrimitive, Copy, Clone, Debug, PartialEq, Eq)]
/// Public only for the sake of errors
pub enum ProtocolConstant {
    Hello = 0x00,

    // NOTE: provide hamster data for as many calls as practical

    // create a function and hand back a token
    // • function
    // • user-args
    // call function
    // and get things back
    // deregister function (comes with data)
    Define,
    Call,
    Result = 0xc5,
    Erroneous = 0x5c,
    Unregister,
    Peek,
    Poke,
    Log,

    // also a version for writeback

    // send key-values (values are byte blobs, or perhaps vectors of serials)
    // read back values?
    // send back stderr / stdout
    Goodbye = 0xff,
}

impl ProtocolConstant {
    pub fn expect(self, expected: ProtocolConstant) -> Result<(), ProtocolError> {
        if self == expected {
            Ok(())
        } else {
            Err(ProtocolError::UnexpectedProtocolConstant {
                got: self,
                expected,
            })
        }
    }
}

#[derive(Debug)]
pub struct FunctionToken(u64);

#[derive(Debug)]
pub struct Request {
    pub command: ProtocolCommand,
    pub aux: Vec<GenericValueBoxed>,
}

#[derive(Debug)]
#[must_use]
pub enum ProtocolCommand {
    Shutdown,
    Define {
        token: FunctionToken,
        function_blob: Vec<u8>,
        associated_data: Vec<GenericValueBoxed>,
    },
    Call {
        token: FunctionToken,
        args: Vec<GenericValueBoxed>,
    },
    Free(FunctionToken),
    Peek(String),
    Poke {
        key: String,
        value: Vec<GenericValueBoxed>,
    },
}

#[derive(Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum LogType {
    Stdout,
    Stderr,
}

#[derive(Debug)]
pub struct LogEntry {
    log_type: LogType,
    line: String,
}

#[derive(Debug)]
pub struct Response<T> {
    pub logs: Vec<LogEntry>,
    pub response_aux: Vec<GenericValueBoxed>,
    pub value: T,
}

/*
 * endpoint specific protocol implementations
 */

impl ControllerProcess {
    pub(crate) fn hello(&mut self) -> io::Result<()> {
        self.write_protocol(ProtocolConstant::Hello)?
            .flush()?
            .read_protocol()?
            .expect(ProtocolConstant::Hello)?;

        Ok(())
    }

    pub fn shutdown(mut self, aux: &[GenericValueRef]) -> io::Result<()> {
        self.write_protocol(ProtocolConstant::Goodbye)?
            .write_generic_vec(aux)?
            .flush()?;
        self.subordinate.wait()?;
        Ok(())
    }

    // create a function and hand back a token
    // • function
    // • user-args
    // call function
    // and get things back
    // deregister function (comes with data)

    fn read_logs(&mut self) -> io::Result<Vec<LogEntry>> {
        let log_ct = self.read_usize()?;
        let mut logs = Vec::with_capacity(log_ct);
        for _ in 0..log_ct {
            let log_type = self.read_log_type()?;
            let line = self.read_string()?;
            logs.push(LogEntry { log_type, line });
        }
        Ok(logs)
    }

    fn read_response<F, V>(&mut self, get_v: F) -> io::Result<Response<V>>
    where
        F: FnOnce(&mut Self) -> io::Result<V>,
    {
        let p = self.flush()?.read_protocol()?;
        match p {
            ProtocolConstant::Result => {
                let logs = self.read_logs()?;
                let response_aux = self.read_generic_vec()?;
                let value = get_v(self)?;
                Ok(Response {
                    logs,
                    response_aux,
                    value,
                })
            }
            ProtocolConstant::Erroneous => {
                let err_type = self.read_err_type()?;
                let logs = self.read_logs()?;
                let aux = self.read_generic_vec()?;
                Err(RemoteError {
                    err_type,
                    logs,
                    aux,
                }
                .into())
            }
            err => Err(ProtocolError::UnexpectedProtocolConstant {
                expected: ProtocolConstant::Result,
                got: err,
            }
            .into()),
        }
    }

    pub fn define_function(
        &mut self,
        function_blob: &[u8],
        associated_data: &[GenericValueRef],
        aux: &[GenericValueRef],
    ) -> io::Result<Response<FunctionToken>> {
        self.id_ctr += 1;
        let token = self.id_ctr;

        self.write_protocol(ProtocolConstant::Define)?
            .write_u64(token)?
            .write_bytes(function_blob)?
            .write_generic_vec(associated_data)?
            .write_generic_vec(aux)?
            .read_response(|_| Ok(FunctionToken(token)))
    }

    pub fn call_function(
        &mut self,
        token: &FunctionToken,
        args: &[GenericValueRef],
        aux: &[GenericValueRef],
    ) -> io::Result<Response<Vec<GenericValueBoxed>>> {
        self.write_protocol(ProtocolConstant::Call)?
            .write_u64(token.0)?
            .write_generic_vec(args)?
            .write_generic_vec(aux)?
            .read_response(|s| s.read_generic_vec())
    }

    pub fn free_function(
        &mut self,
        token: &FunctionToken,
        aux: &[GenericValueRef],
    ) -> io::Result<Response<()>> {
        self.write_protocol(ProtocolConstant::Unregister)?
            .write_u64(token.0)?
            .write_generic_vec(aux)?
            .read_response(|_| Ok(()))
    }

    pub fn peek(
        &mut self,
        key: &str,
        aux: &[GenericValueRef],
    ) -> io::Result<Response<Vec<GenericValueBoxed>>> {
        self.write_protocol(ProtocolConstant::Peek)?
            .write_string(key)?
            .write_generic_vec(aux)?
            .read_response(|s| s.read_generic_vec())
    }

    pub fn poke(
        &mut self,
        key: &str,
        value: &[GenericValueRef],
        aux: &[GenericValueRef],
    ) -> io::Result<Response<()>> {
        self.write_protocol(ProtocolConstant::Poke)?
            .write_string(key)?
            .write_generic_vec(value)?
            .write_generic_vec(aux)?
            .read_response(|_| Ok(()))
    }
}

impl SubordinateProcess {
    pub(crate) fn hello(&mut self) -> io::Result<()> {
        self.read_protocol()?.expect(ProtocolConstant::Hello)?;
        self.write_protocol(ProtocolConstant::Hello)?.flush()?;
        Ok(())
    }

    fn recv_define(&mut self) -> io::Result<ProtocolCommand> {
        let token = self.read_u64()?;
        let function_blob = self.read_bytes()?;
        let associated_data = self.read_generic_vec()?;

        Ok(ProtocolCommand::Define {
            token: FunctionToken(token),
            function_blob,
            associated_data,
        })
    }

    fn recv_call(&mut self) -> io::Result<ProtocolCommand> {
        let token = self.read_u64()?;
        let args = self.read_generic_vec()?;
        Ok(ProtocolCommand::Call {
            token: FunctionToken(token),
            args,
        })
    }

    fn recv_free(&mut self) -> io::Result<ProtocolCommand> {
        let token = self.read_u64()?;
        Ok(ProtocolCommand::Free(FunctionToken(token)))
    }

    fn recv_peek(&mut self) -> io::Result<ProtocolCommand> {
        let key = self.read_string()?;
        Ok(ProtocolCommand::Peek(key))
    }

    fn recv_poke(&mut self) -> io::Result<ProtocolCommand> {
        let key = self.read_string()?;
        let value = self.read_generic_vec()?;
        Ok(ProtocolCommand::Poke { key, value })
    }

    pub fn recv_command(&mut self) -> io::Result<Request> {
        let command = match self.read_protocol()? {
            ProtocolConstant::Define => self.recv_define(),
            ProtocolConstant::Call => self.recv_call(),
            ProtocolConstant::Unregister => self.recv_free(),

            ProtocolConstant::Peek => self.recv_peek(),
            ProtocolConstant::Poke => self.recv_poke(),

            ProtocolConstant::Goodbye => Ok(ProtocolCommand::Shutdown),
            err => io::Result::Err(ProtocolError::InappropriateProtocolConstant(err).into()),
        }?;

        let aux = self.read_generic_vec()?;

        Ok(Request { command, aux })
    }

    fn respond<F>(&mut self, aux: &[GenericValueRef], value_writer: F) -> io::Result<()>
    where
        F: FnOnce(&mut Self) -> io::Result<&mut Self>,
    {
        self.write_protocol(ProtocolConstant::Result)?;

        //TODO: we'll need to read logs, for now zero logs
        self.write_usize(0)?;

        self.write_generic_vec(aux)?;
        value_writer(self)?.flush()?;
        Ok(())
    }

    pub fn respond_to_define(&mut self, aux: &[GenericValueRef]) -> io::Result<()> {
        self.respond(aux, |s| Ok(s))
    }

    pub fn respond_to_call(
        &mut self,
        call_return: &[GenericValueRef],
        aux: &[GenericValueRef],
    ) -> io::Result<()> {
        self.respond(aux, |s| s.write_generic_vec(call_return))
    }

    pub fn respond_to_unregister(&mut self, aux: &[GenericValueRef]) -> io::Result<()> {
        self.respond(aux, |s| Ok(s))
    }

    pub fn respond_to_peek(
        &mut self,
        peek_value: &[GenericValueRef],
        aux: &[GenericValueRef],
    ) -> io::Result<()> {
        self.respond(aux, |s| s.write_generic_vec(peek_value))
    }

    pub fn respond_to_poke(&mut self, aux: &[GenericValueRef]) -> io::Result<()> {
        self.respond(aux, |s| Ok(s))
    }

    pub fn respond_with_error(
        &mut self,
        error_type: RemoteErrorType,
        aux: &[GenericValueRef],
    ) -> io::Result<()> {
        self.write_protocol(ProtocolConstant::Erroneous)?
            .write_err_type(error_type)?
            .write_generic_vec(aux)?
            .flush()?;
        Ok(())
    }
}
