//! Parse responses from HAProxy sockets.

use crate::errors::Error;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum Level {
    Admin,
    Operator,
    User,
}

impl FromStr for Level {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "admin" => Ok(Level::Admin),
            "operator" => Ok(Level::Operator),
            "user" => Ok(Level::User),
            _ => Err(Error::ParseFailure),
        }
    }
}

#[derive(Debug, Hash, Eq, PartialEq)]
pub struct CliSocket {
    pub socket: CliSocketAddr,
    pub level: Level,
    pub processes: CliSocketProcesses,
}

impl FromStr for CliSocket {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(3, ' ').collect();
        match parts.as_slice() {
            [socket_addr, level, processes] => Ok(CliSocket {
                socket: CliSocketAddr::from_str(socket_addr)?,
                level: Level::from_str(level)?,
                processes: CliSocketProcesses::from_str(processes)?,
            }),
            _ => Err(Error::ParseFailure),
        }
    }
}

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum CliSocketAddr {
    Unix(PathBuf),
    Ip(std::net::SocketAddr),
    SocketPair(String),
    /// Abstract socket address (see `man 7 unix`).
    AbstractSocket(String),
    /// The HAProxy implementation uses "unknown" as a catchall in its formatter, so we
    /// support that here.
    Unknown,
}

impl FromStr for CliSocketAddr {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(2, '@').collect();
        match parts.as_slice() {
            ["unknown"] => Ok(CliSocketAddr::Unknown),
            ["unix", path] => Ok(CliSocketAddr::Unix(path.into())),
            ["ipv4", addr] => {
                let socket_addr =
                    std::net::SocketAddrV4::from_str(addr).map_err(|_| Error::ParseFailure)?;
                Ok(CliSocketAddr::Ip(socket_addr.into()))
            }
            ["ipv6", addr] => {
                let socket_addr =
                    std::net::SocketAddrV6::from_str(addr).map_err(|_| Error::ParseFailure)?;
                Ok(CliSocketAddr::Ip(socket_addr.into()))
            }
            ["sockpair", addr] => Ok(CliSocketAddr::SocketPair(addr.to_string())),
            ["abns", addr] => Ok(CliSocketAddr::AbstractSocket(addr.to_string())),
            _ => Err(Error::ParseFailure),
        }
    }
}

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum CliSocketProcesses {
    All,
    List(Vec<u32>),
}

impl FromStr for CliSocketProcesses {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "all" {
            Ok(CliSocketProcesses::All)
        } else {
            let processes = s
                .split(',')
                .map(|process| process.parse().map_err(|_| Error::ParseFailure))
                .collect::<Result<_, Self::Err>>()?;
            Ok(CliSocketProcesses::List(processes))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_from_bytes() {
        assert_eq!(Level::from_str("admin").unwrap(), Level::Admin);
        assert_eq!(Level::from_str("operator").unwrap(), Level::Operator);
        assert_eq!(Level::from_str("user").unwrap(), Level::User);
        Level::from_str("1234").expect_err("Parsed invalid level");
    }

    #[test]
    fn cli_socket_from_str() {
        assert_eq!(
            CliSocket::from_str("unix@/var/run/haproxy.sock admin all").unwrap(),
            CliSocket {
                socket: CliSocketAddr::Unix("/var/run/haproxy.sock".into()),
                level: Level::Admin,
                processes: CliSocketProcesses::All,
            }
        )
    }

    #[test]
    fn cli_socket_addr_from_str() {
        assert_eq!(
            CliSocketAddr::from_str("unix@/var/run/haproxy.sock").unwrap(),
            CliSocketAddr::Unix("/var/run/haproxy.sock".into())
        );
        assert_eq!(
            CliSocketAddr::from_str("ipv4@127.0.0.1:9999").unwrap(),
            CliSocketAddr::Ip("127.0.0.1:9999".parse().unwrap())
        );
        assert_eq!(
            CliSocketAddr::from_str("ipv6@[::]:9999").unwrap(),
            CliSocketAddr::Ip("[::]:9999".parse().unwrap())
        );
        assert_eq!(
            CliSocketAddr::from_str("sockpair@1234").unwrap(),
            CliSocketAddr::SocketPair("1234".into())
        );
        assert_eq!(
            CliSocketAddr::from_str("abns@abcd").unwrap(),
            CliSocketAddr::AbstractSocket("abcd".into())
        );
        assert_eq!(
            CliSocketAddr::from_str("unknown").unwrap(),
            CliSocketAddr::Unknown
        );
    }

    #[test]
    fn cli_socket_processes_from_str() {
        assert_eq!(
            CliSocketProcesses::from_str("all").unwrap(),
            CliSocketProcesses::All,
        );
        assert_eq!(
            CliSocketProcesses::from_str("0,1,2").unwrap(),
            CliSocketProcesses::List(vec![0, 1, 2]),
        );
    }
}