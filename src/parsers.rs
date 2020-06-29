use std::io::{BufRead, BufReader, Read, Write};
use std::str::FromStr;

use crate::errors::Error;
use crate::responses::CliSocket;

pub fn parse_cli_sockets<T: Read + Write>(
    reader: &mut BufReader<T>,
) -> Result<Vec<CliSocket>, Error> {
    reader
        .lines()
        // Filter out lines starting with '#', preserving any errors.
        .filter(|line_res| {
            !line_res
                .as_ref()
                .map(|line| line == "" || line.starts_with('#'))
                .unwrap_or(true)
        })
        // Convert io::Error to Error. In the Ok case, pass the line to CliSocket::from_str.
        .map(|line_res| {
            line_res
                .map_err(|err| Error::from(err))
                .and_then(|line| CliSocket::from_str(line.as_str()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{responses, ConnectionBuilder, UnixSocketBuilder};

    #[test]
    fn connection_cli_sockets() {
        let builder = UnixSocketBuilder::new("/tmp/socket/haproxy.sock".into());
        let connection = builder.connect().unwrap();
        let sockets = connection.cli_sockets().unwrap();

        assert_eq!(sockets.len(), 3);
        assert_eq!(
            sockets[0],
            responses::CliSocket {
                socket: responses::CliSocketAddr::Unix("/var/run/haproxy.sock".into()),
                level: responses::Level::Admin,
                processes: responses::CliSocketProcesses::All
            }
        );
        assert_eq!(
            sockets[1],
            responses::CliSocket {
                socket: responses::CliSocketAddr::Ip("127.0.0.1:9999".parse().unwrap()),
                level: responses::Level::Admin,
                processes: responses::CliSocketProcesses::All
            }
        );
        assert_eq!(
            sockets[2],
            responses::CliSocket {
                socket: responses::CliSocketAddr::Ip("[::]:9999".parse().unwrap()),
                level: responses::Level::Admin,
                processes: responses::CliSocketProcesses::All
            }
        );
    }
}
