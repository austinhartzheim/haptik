use crate::errors::Error;
use std::fmt::{self, Display};
use std::str::FromStr;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct AclEntry<T> {
    /// Element pointer within HAProxy. Forcing u64 instead of usize to allow running haptik on
    /// 32-bit machines when HAProxy pointers are 64-bits.
    pub id: u64,
    /// The value of the ACL entry.
    pub value: T,
}

impl<T: FromStr> FromStr for AclEntry<T> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(2, ' ').collect();
        if let [id, value] = parts.as_slice() {
            if id.starts_with("0x") {
                Ok(Self {
                    id: u64::from_str_radix(&id[2..], 16)?,
                    value: T::from_str(value).map_err(|_| Error::ParseFailure)?,
                })
            } else {
                Err(Error::ParseFailure)
            }
        } else {
            Err(Error::ParseFailure)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AclId {
    Id(i32),
}

impl Display for AclId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AclId::Id(id) => f.write_fmt(format_args!("#{}", id)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aclentry_from_str() {
        assert_eq!(
            AclEntry::<std::net::IpAddr>::from_str("0x1234 127.0.0.1").unwrap(),
            AclEntry::<std::net::IpAddr> {
                id: 0x1234,
                value: "127.0.0.1".parse().unwrap()
            }
        );
    }
}
