//! Parse responses from HAProxy sockets.

use crate::errors::Error;
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
            "admin\n" => Ok(Level::Admin),
            "operator\n" => Ok(Level::Operator),
            "user\n" => Ok(Level::User),
            _ => Err(Error::ParseFailure),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_from_bytes() {
        assert_eq!(Level::from_str("admin\n").unwrap(), Level::Admin);
        assert_eq!(Level::from_str("operator\n").unwrap(), Level::Operator);
        assert_eq!(Level::from_str("user\n").unwrap(), Level::User);
        Level::from_str("1234\n").expect_err("Parsed invalid level");
    }
}
