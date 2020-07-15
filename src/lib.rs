//! Issue commands to an HAProxy over a stats socket.

#![forbid(unsafe_code)]

use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::str::FromStr;

mod commands;
pub mod errors;
pub mod models;
mod parsers;
pub mod requests;
pub mod responses;

use errors::Error;
use requests::{AclId, BackendId, ErrorFlag};
use responses::Acl;

/// Support connections to HAProxy via Unix sockets and TCP sockets using the same interface.
pub trait ConnectionBuilder {
    type Connection;

    /// Create a new connection to HAProxy.
    fn connect(&self) -> Result<Self::Connection, io::Error>;
}

/// Configuration for connecting to an HAProxy Unix Socket.
///
/// This allows configuration of the path for the Unix socket.
#[derive(Clone, Debug, Eq, PartialEq)]
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

impl From<PathBuf> for UnixSocketBuilder {
    fn from(path: PathBuf) -> Self {
        Self { path }
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
    /// Add an entry to an HAProxy ACL.
    ///
    /// HAProxy's `add acl` command does not support entries with spaces, so this command truncates
    /// the value at the first space.
    ///
    /// # Examples
    /// ```no_run
    /// use std::net::Ipv4Addr;
    /// use haptik::{ConnectionBuilder, UnixSocketBuilder};
    /// use haptik::requests::AclId;
    ///
    /// let socket_builder = UnixSocketBuilder::default();
    /// let connection = socket_builder.connect().expect("Failed to connect");
    /// connection.acl_add(AclId::Id(0), Ipv4Addr::new(127, 0, 0, 1));
    /// ```
    pub fn acl_add<E: ToString>(mut self, id: AclId, value: E) -> Result<(), Error> {
        let string = value.to_string();
        let parts: Vec<&str> = string.splitn(2, ' ').collect();

        commands::add_acl(&mut self.socket, id, parts[0])?;
        commands::end(&mut self.socket)?;

        parsers::parse_acl_add(&mut self.reader)
    }

    /// Query HAProxy for the contents of an ACL.
    ///
    /// ACLs in HAProxy support multiple types of data (strings, IP addresses, etc.); but the type
    /// data is not immediately available when querying the ACL. If you know the underlying type,
    /// you can instruct `haptik` to parse the ACL entries into that type so long as it implements
    /// `FromStr`. Provide the type as the type parameter to this method. If the type is unknown,
    /// you can use `String`.
    ///
    /// # Examples
    /// ```no_run
    /// use std::net::IpAddr;
    /// use haptik::{ConnectionBuilder, UnixSocketBuilder};
    /// use haptik::requests::AclId;
    ///
    /// let socket_builder = UnixSocketBuilder::default();
    /// let connection = socket_builder.connect().expect("Failed to connect");
    /// let acl_data = connection.acl_data::<IpAddr>(AclId::Id(0)).expect("Failed to query ACL");
    /// for acl_entry in acl_data.iter() {
    ///     println!("ACL Entry: id={}, value={}", acl_entry.id, acl_entry.value);
    /// }
    /// ```
    pub fn acl_data<E: FromStr>(mut self, id: AclId) -> Result<Vec<models::AclEntry<E>>, Error> {
        commands::show_acl_entries(&mut self.socket, id)?;
        commands::end(&mut self.socket)?;

        parsers::parse_acl_entries(&mut self.reader)
    }

    pub fn acl_list(mut self) -> Result<Vec<Acl>, Error> {
        commands::show_acl(&mut self.socket)?;
        commands::end(&mut self.socket)?;

        parsers::parse_acl_list(&mut self.reader)
    }

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

    /// Query HAProxy for the error count of all backends and all error types.
    ///
    /// This command is identical to `errors_backend(BackendId::All, ErrorFlag::All)`.
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

    /// Query HAProxy for the error count of a specific backend and a specific error type.
    ///
    /// Passing `BackendId::All` queries errors for all backends; this is the same as passing `-1`
    /// as the ID, which HAProxy interprets as all.
    ///
    /// # Examples
    /// ```no_run
    /// use haptik::{ConnectionBuilder, UnixSocketBuilder};
    /// use haptik::requests::{BackendId, ErrorFlag};
    /// use haptik::responses::Level;
    ///
    /// let socket_builder = UnixSocketBuilder::default();
    /// let connection = socket_builder.connect().expect("Failed to connect");
    /// assert_eq!(
    ///     connection.errors_backend(BackendId::Id(1), ErrorFlag::All)
    ///         .expect("Failed to query error count"),
    ///     0
    /// );
    /// ```
    pub fn errors_backend(
        mut self,
        backend: BackendId,
        error_type: ErrorFlag,
    ) -> Result<u32, Error> {
        commands::show_errors_backend(&mut self.socket, backend, error_type)?;
        commands::end(&mut self.socket)?;

        parsers::parse_errors(&mut self.reader)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
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
    #[ignore]
    fn connection_acl_add() {
        let ip = std::net::Ipv4Addr::new(255, 255, 255, 255);
        let acl_id = AclId::Id(1);

        let builder = UnixSocketBuilder::new("/tmp/socket/haproxy.sock".into());
        let connection = builder.connect().unwrap();
        connection.acl_add(acl_id, &ip).unwrap();

        // Check that the ACL contains the new entry
        let connection = builder.connect().unwrap();
        let acl_data = connection.acl_data::<std::net::Ipv4Addr>(acl_id).unwrap();
        assert!(acl_data.iter().any(|entry| entry.value == ip))
    }

    #[test]
    #[ignore]
    fn connection_acl_data() {
        let builder = UnixSocketBuilder::new("/tmp/socket/haproxy.sock".into());
        let connection = builder.connect().unwrap();
        let acl_data = connection
            .acl_data::<std::net::IpAddr>(AclId::Id(0))
            .unwrap();
        assert_eq!(acl_data.len(), 2);
        assert_eq!(
            acl_data[0].value,
            std::net::IpAddr::V4("127.0.0.1".parse().unwrap())
        );
        assert_eq!(
            acl_data[1].value,
            std::net::IpAddr::V4("127.0.0.2".parse().unwrap())
        );
    }

    #[test]
    #[ignore]
    fn connection_acl_data_str() {
        let builder = UnixSocketBuilder::new("/tmp/socket/haproxy.sock".into());
        let connection = builder.connect().unwrap();
        let acl_data = connection.acl_data::<String>(AclId::Id(0)).unwrap();
        assert_eq!(acl_data.len(), 2);
        assert_eq!(acl_data[0].value, "127.0.0.1",);
        assert_eq!(acl_data[1].value, "127.0.0.2",);
    }

    #[test]
    #[ignore]
    fn connection_acl_list() {
        let builder = UnixSocketBuilder::new("/tmp/socket/haproxy.sock".into());
        let connection = builder.connect().unwrap();
        let acls = connection.acl_list().unwrap();

        assert_eq!(acls.len(), 2);

        assert_eq!(acls[0].id, 0);
        assert_eq!(acls[0].reference, None);
        assert_eq!(
            acls[0].description,
            "acl 'src' file '/usr/local/etc/haproxy/haproxy.cfg' line 20"
        );
        assert_eq!(acls[1].id, 1);
        assert_eq!(acls[1].reference, None);
        assert_eq!(
            acls[1].description,
            "acl 'src' file '/usr/local/etc/haproxy/haproxy.cfg' line 21"
        );
    }

    #[test]
    #[ignore]
    fn connection_cli_sockets() {
        let builder = UnixSocketBuilder::new("/tmp/socket/haproxy.sock".into());
        let connection = builder.connect().unwrap();
        let sockets = connection.cli_sockets().unwrap();

        assert_eq!(sockets.len(), 3);
        assert_eq!(
            sockets[0],
            responses::CliSocket {
                address: responses::CliSocketAddr::Unix("/var/run/haproxy.sock".into()),
                level: responses::Level::Admin,
                processes: responses::CliSocketProcesses::All
            }
        );
        assert_eq!(
            sockets[1],
            responses::CliSocket {
                address: responses::CliSocketAddr::Ip("127.0.0.1:9999".parse().unwrap()),
                level: responses::Level::Admin,
                processes: responses::CliSocketProcesses::All
            }
        );
        assert_eq!(
            sockets[2],
            responses::CliSocket {
                address: responses::CliSocketAddr::Ip("[::]:9999".parse().unwrap()),
                level: responses::Level::Admin,
                processes: responses::CliSocketProcesses::All
            }
        );
    }

    #[test]
    #[ignore]
    fn connection_level() {
        let builder = UnixSocketBuilder::new("/tmp/socket/haproxy.sock".into());
        let connection = builder.connect().unwrap();
        assert_eq!(connection.level().unwrap(), responses::Level::Admin);
    }

    #[test]
    #[ignore]
    fn connection_errors() {
        let builder = UnixSocketBuilder::new("/tmp/socket/haproxy.sock".into());
        let connection = builder.connect().unwrap();
        assert_eq!(connection.errors().unwrap(), 0);
    }

    #[test]
    #[ignore]
    fn connection_errors_backend_all() {
        let builder = UnixSocketBuilder::new("/tmp/socket/haproxy.sock".into());
        let connection = builder.connect().unwrap();
        assert_eq!(
            connection
                .errors_backend(BackendId::All, ErrorFlag::All)
                .unwrap(),
            0
        );
    }

    #[test]
    #[ignore]
    fn connection_errors_backend_id() {
        let builder = UnixSocketBuilder::new("/tmp/socket/haproxy.sock".into());
        let connection = builder.connect().unwrap();
        assert_eq!(
            connection
                .errors_backend(BackendId::Id(1), ErrorFlag::Request)
                .unwrap(),
            0
        );
    }
}
