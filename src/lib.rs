//! Issue commands to an HAProxy over a stats socket.
//!
//! # Connecting to HAProxy
//! Connect to HAProxy using either a Unix socket or a TCP socket. By convention, sockets are
//! closed after each request, so you will likely want a to choose a `ConnectionBuilder` to spawn
//! connections for you.
//!
//! ```no_run
//! // Connect using a Unix socket
//! use haptik::{ConnectionBuilder, UnixSocketBuilder};
//! let connection_builder = UnixSocketBuilder::new("/var/lib/haproxy.sock");
//! let connection = connection_builder.connect().expect("Failed connecting to HAProxy");
//! ```
//! ```no_run
//! // Connect using a TCP socket
//! use haptik::{ConnectionBuilder, TcpSocketBuilder};
//! use std::net::{SocketAddr, Ipv4Addr};
//! let connection_builder = TcpSocketBuilder::new(
//!     SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 9999)
//! );
//! let connection = connection_builder.connect().expect("Failed to connect to HAProxy");
//! ```
//!
//! Alternatively, [`UnixSocketBuilder`] and [`TcpSocketBuilder`] provide default implementations
//! defaulting to `/var/lib/haproxy.sock` and `127.0.0.1:9999` respectively.
//! ```
//! # use haptik::{UnixSocketBuilder, TcpSocketBuilder};
//! let connection_builder = UnixSocketBuilder::default();
//! let connection_builder = TcpSocketBuilder::default();
//! ```
//!
//! # Issuing Commands
//! Calling `.connect()` on a [`ConnectionBuilder`] yields a [`Connection`] you can use to issue
//! a single command to HAProxy. Generally, you will interact with an abstraction over a
//! [`ConnectionBuilder`] which will spawn connections as needed. However, you can interact
//! directly with the connection object if desired.
//!
//! ```no_run
//! # use haptik::{ConnectionBuilder, UnixSocketBuilder};
//! let connection_builder = UnixSocketBuilder::default();
//! let connection = connection_builder.connect().expect("Failed to connect to HAProxy");
//! let backend_error_count = connection.errors().expect("Failed to query backend error count");
//! println!("Total errors across all backends: {}", backend_error_count);
//! ```

#![forbid(unsafe_code)]

mod commands;
pub mod connection;
pub mod errors;
pub mod models;
mod parsers;
pub mod requests;
pub mod responses;

pub use connection::{Connection, ConnectionBuilder, TcpSocketBuilder, UnixSocketBuilder};
