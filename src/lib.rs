//! Issue commands to an HAProxy over a stats socket.

#![forbid(unsafe_code)]

mod commands;
pub mod connection;
pub mod errors;
pub mod models;
mod parsers;
pub mod requests;
pub mod responses;

pub use connection::{Connection, ConnectionBuilder, UnixSocketBuilder};
