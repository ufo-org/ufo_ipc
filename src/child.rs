use std::io::Result;

use ufo_ipc::*;

fn main() -> Result<()> {
    let mut subordinate = subordinate_begin()?;

    'shutdown: loop {
        let request = subordinate.recv_command()?;

        match request.command {
            ProtocolCommand::Peek(key) => subordinate.respond_to_peek(
                &[GenericValue::Vstring("test response")],
                &[GenericValue::Vstring(&key)],
            )?,
            ProtocolCommand::Poke { key, value } => subordinate.respond_to_poke(&[
                GenericValue::Vstring(&key), GenericValue::Vstring(value[0].expect_string()?), 
            ])?,

            ProtocolCommand::Shutdown => break 'shutdown,
            _ => subordinate.respond_with_error(RemoteErrorType::ProtocolError, &[])?,
        }
    }

    Ok(())
}
