use std::io::{Result, Write};

pub fn end<W: Write>(w: &mut W) -> Result<()> {
    w.write_all(b"\n")
}

pub fn show_cli_level<W: Write>(w: &mut W) -> Result<()> {
    w.write_all(b"show cli level")
}

pub fn show_cli_sockets<W: Write>(w: &mut W) -> Result<()> {
    w.write_all(b"show cli sockets")
}

pub fn show_errors<W: Write>(w: &mut W) -> Result<()> {
    w.write_all(b"show errors")
}
