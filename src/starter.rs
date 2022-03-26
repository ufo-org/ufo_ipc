use std::{io::Result, process::Command};

use ufo_ipc::*;

fn main() -> Result<()> {
    let mut child = Command::new("cargo")
        .args(&["run", "--bin", "child"])
        .start_subordinate_process()?;

    let response = child.peek("test", &[])?;
    println!("peek: {:?}", response);

    let response = child.poke("test", &[GenericValue::Vstring("testing")], &[])?;
    println!("poke: {:?}", response);

    child.shutdown(&[])?;

    Ok(())
}
