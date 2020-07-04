use std::io::{self, BufRead, BufReader, Read};
use std::str::FromStr;

use crate::errors::Error;
use crate::responses::{Acl, CliSocket};

pub fn parse_acl_list<T: Read>(reader: &mut BufReader<T>) -> Result<Vec<Acl>, Error> {
    skip_comment_or_empty_lines(reader.lines())
        .map(|line_res| {
            line_res
                .map_err(Error::from)
                .and_then(|line| Acl::from_str(line.as_str()))
        })
        .collect()
}

pub fn parse_cli_sockets<T: Read>(reader: &mut BufReader<T>) -> Result<Vec<CliSocket>, Error> {
    skip_comment_or_empty_lines(reader.lines())
        // Convert io::Error to Error. In the Ok case, pass the line to CliSocket::from_str.
        .map(|line_res| {
            line_res
                .map_err(Error::from)
                .and_then(|line| CliSocket::from_str(line.as_str()))
        })
        .collect()
}

pub fn parse_errors<T: Read>(reader: &mut BufReader<T>) -> Result<u32, Error> {
    let mut buf = String::with_capacity(65);
    reader.read_line(&mut buf)?;
    buf.pop(); // Remove trailing '\n'

    buf.rsplitn(2, ' ')
        .next()
        .ok_or(Error::ParseFailure)
        .and_then(|count| u32::from_str(count).map_err(|_| Error::ParseFailure))
}

/// Skip lines starting with '#' and any line that is empty.
fn skip_comment_or_empty_lines<B: io::BufRead>(
    lines: io::Lines<B>,
) -> impl Iterator<Item = Result<String, io::Error>> {
    lines.filter(|line_res| {
        !line_res
            .as_ref()
            .map(|line| line == "" || line.starts_with('#'))
            .unwrap_or(true)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::responses;

    #[test]
    fn parse_acl_list_valid_input() {
        let mut buffer = BufReader::new(&b"# id (file) description\n0 () acl 'src' file '/usr/local/etc/haproxy/haproxy.cfg' line 20"[..]);
        assert_eq!(parse_acl_list(&mut buffer).unwrap().len(), 1);
    }

    #[test]
    fn parse_cli_sockets_valid_input() {
        let mut buffer = BufReader::new(
            &b"unix@/var/run/haproxy.sock admin all\nipv4@127.0.0.1:9999 user all\n\n"[..],
        );
        let sockets = parse_cli_sockets(&mut buffer).expect("Failed to parse valid input");

        assert_eq!(sockets.len(), 2);
        assert_eq!(
            sockets[0],
            CliSocket {
                address: responses::CliSocketAddr::Unix("/var/run/haproxy.sock".into()),
                level: responses::Level::Admin,
                processes: responses::CliSocketProcesses::All
            }
        );
        assert_eq!(
            sockets[1],
            CliSocket {
                address: responses::CliSocketAddr::Ip("127.0.0.1:9999".parse().unwrap()),
                level: responses::Level::User,
                processes: responses::CliSocketProcesses::All
            }
        );
    }

    #[test]
    fn parse_errors_valid_input() {
        let mut buffer =
            BufReader::new(&b"Total events captured on [01/Jan/2020:03:15:05.071] : 0\n"[..]);
        assert_eq!(parse_errors(&mut buffer).unwrap(), 0);

        let mut buffer =
            BufReader::new(&b"Total events captured on [01/Jan/2020:03:15:05.071] : 100\n"[..]);
        assert_eq!(parse_errors(&mut buffer).unwrap(), 100);
    }
}
