//! Format commands.

use crate::requests::{AclId, BackendId, ErrorFlag};
use std::io::{Result, Write};

pub fn end<W: Write>(w: &mut W) -> Result<()> {
    w.write_all(b"\n")
}

pub fn show_acl<W: Write>(w: &mut W) -> Result<()> {
    w.write_all(b"show acl")
}

pub fn show_acl_entries<W: Write>(w: &mut W, id: AclId) -> Result<()> {
    w.write_fmt(format_args!("show acl {}", id))
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

pub fn show_errors_backend<W: Write>(
    w: &mut W,
    id: BackendId,
    error_type: ErrorFlag,
) -> Result<()> {
    let error_type_str = match error_type {
        ErrorFlag::All => "",
        ErrorFlag::Request => " request",
        ErrorFlag::Response => " response",
    };
    w.write_fmt(format_args!("show errors {}{}", id, error_type_str))
}
