//! Request types for HAProxy.

use std::fmt::{self, Display};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BackendId<'a> {
    /// Match all backends.
    All,
    /// Match a backend ID.
    Id(i32),
    /// Match a backend name.
    Name(&'a str),
}

impl Display for BackendId<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BackendId::All => f.write_str("-1"),
            BackendId::Name(name) => f.write_str(name),
            BackendId::Id(id) => id.fmt(f),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorFlag {
    /// Match request and response errors.
    All,
    /// Match only request errors.
    Request,
    /// Match only response errors.
    Response,
}
