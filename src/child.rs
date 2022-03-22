use std::io::Result;

use ufo_ipc::*;

fn main() -> Result<()> {
    let mut subordinate = subordinate_begin()?;

    subordinate
        .write_string("test!")?
        .write_protocol(ProtocolConstant::Goodbye)?;

    Ok(())
}
