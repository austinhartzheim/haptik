//! Issue commands to an HAProxy over a stats socket.

#![feature(result_flattening)]

use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::str::FromStr;

mod commands;
pub mod errors;
mod parsers;
pub mod responses;

use errors::Error;

/// Support connections to HAProxy via Unix sockets and TCP sockets using the same interface.
pub trait ConnectionBuilder {
    type Connection;

    /// Create a new connection to HAProxy.
    fn connect(&self) -> Result<Self::Connection, io::Error>;
}

/// Configuration for connecting to an HAProxy Unix Socket.
///
/// This allows configuration of the path for the Unix socket.
pub struct UnixSocketBuilder {
    /// The path of the Unix socket.
    path: PathBuf,
}

impl UnixSocketBuilder {
    /// Create a new `UnixSocketBuilder` to establish connections to HAProxy via Unix Socket.
    ///
    /// # Examples
    /// ```no_run
    /// use haptik::{ConnectionBuilder, UnixSocketBuilder};
    ///
    /// let socket_builder = UnixSocketBuilder::new("/var/run/haproxy.sock".into());
    /// let connection = socket_builder.connect().expect("Failed to connect");
    /// ```
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

/// Use a default location of `/var/run/haproxy.sock` for the Unix socket.
impl Default for UnixSocketBuilder {
    fn default() -> Self {
        Self {
            path: PathBuf::from("/var/run/haproxy.sock"),
        }
    }
}

impl ConnectionBuilder for UnixSocketBuilder {
    type Connection = Connection<UnixStream>;

    fn connect(&self) -> Result<Self::Connection, io::Error> {
        let socket = UnixStream::connect(&self.path)?;
        let reader = BufReader::new(socket.try_clone()?);

        Ok(Connection { socket, reader })
    }
}

/// A connection to HAProxy via any of the supported transports.
///
/// By convention, connections are closed after each command. Therefore, many of the methods on
/// `Connection` take `self` to force destruction of the `Connection` instance after use. Use a
/// `ConnectionBuilder` to create connections for each use.
#[derive(Debug)]
pub struct Connection<T> {
    socket: T,
    reader: BufReader<T>,
}

impl<T: Read + Write> Connection<T> {
    /// Query HAProxy to determine the current level.
    ///
    /// # Examples
    /// ```no_run
    /// use haptik::{ConnectionBuilder, UnixSocketBuilder};
    /// use haptik::responses::Level;
    ///
    /// let socket_builder = UnixSocketBuilder::default();
    /// let connection = socket_builder.connect().expect("Failed to connect");
    /// assert_eq!(connection.level().expect("Failed to query level"), Level::Admin);
    /// ```
    pub fn level(mut self) -> Result<responses::Level, Error> {
        commands::show_cli_level(&mut self.socket)?;
        commands::end(&mut self.socket)?;

        let mut buf = String::new();
        self.reader.read_line(&mut buf)?;
        buf.pop(); // Remove trailing '\n'

        responses::Level::from_str(buf.as_str())
    }

    /// Query HAProxy for the list of configured CLI sockets.
    ///
    /// # Examples
    /// ```no_run
    /// use haptik::{ConnectionBuilder, UnixSocketBuilder};
    /// use haptik::responses::Level;
    ///
    /// let socket_builder = UnixSocketBuilder::default();
    /// let connection = socket_builder.connect().expect("Failed to connect");
    /// println!("{:?}", connection.cli_sockets().expect("Failed to query CLI sockets"));
    /// ```
    pub fn cli_sockets(mut self) -> Result<Vec<responses::CliSocket>, Error> {
        commands::show_cli_sockets(&mut self.socket)?;
        commands::end(&mut self.socket)?;

        parsers::parse_cli_sockets(&mut self.reader)
    }

    /// Query HAProxy for the error count.
    ///
    /// # Examples
    /// ```no_run
    /// use haptik::{ConnectionBuilder, UnixSocketBuilder};
    /// use haptik::responses::Level;
    ///
    /// let socket_builder = UnixSocketBuilder::default();
    /// let connection = socket_builder.connect().expect("Failed to connect");
    /// assert_eq!(connection.errors().expect("Failed to query error count"), 0);
    /// ```
    pub fn errors(mut self) -> Result<u32, Error> {
        commands::show_errors(&mut self.socket)?;
        commands::end(&mut self.socket)?;

        parsers::parse_errors(&mut self.reader)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix_socket_builder_connects() {
        let builder = UnixSocketBuilder::new("/tmp/socket/haproxy.sock".into());
        assert!(
            builder.connect().is_ok(),
            "Failed to connect to the HAProxy Unix socket"
        );
    }

    #[test]
    fn unix_socket_builder_errors_on_invalid_socket() {
        let builder = UnixSocketBuilder::new("/tmp/invalid.sock".into());
        assert_eq!(
            builder
                .connect()
                .expect_err("Connected to a Unix socket that does not exist")
                .kind(),
            io::ErrorKind::NotFound
        );
    }

    #[test]
    fn connection_level() {
        let builder = UnixSocketBuilder::new("/tmp/socket/haproxy.sock".into());
        let connection = builder.connect().unwrap();
        assert_eq!(connection.level().unwrap(), responses::Level::Admin);
    }
}