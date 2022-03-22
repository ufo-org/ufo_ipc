use std::{io::Result, process::Command};

use ufo_ipc::*;

fn main() -> Result<()> {
    let mut child = Command::new("cargo")
        .args(&["run", "--bin", "child"])
        .start_subordinate_process()?;

    println!("Got: {}", child.read_string()?);

    child.read_protocol()?.expect(ProtocolConstant::Goodbye)?;

    Ok(())
}
