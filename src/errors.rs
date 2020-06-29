//! Error types.

#[derive(Debug)]
pub enum Error {
    /// Failure parsing response from HAProxy.
    ParseFailure,

    /// Error encountered while performing IO.
    IoError(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
}