use serde_derive::{Deserialize, Serialize};
use std::fmt;


pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Error {
    Internal(String),

    Abort
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        match self {
            Error::Internal(s) => {
                write!(f, "{}", s)
            }
            Error::Abort => write!(f, "Operation aborted"),
        }
    }
}

impl From<Box<bincode::ErrorKind>> for Error {
    fn from(err: Box<bincode::ErrorKind>) -> Self {
        Error::Internal(err.to_string())
    }
}

impl From<std::array::TryFromSliceError> for Error {
    fn from(err: std::array::TryFromSliceError) -> Self {
        Error::Internal(err.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Internal(err.to_string())
    }
}

impl From<std::net::AddrParseError> for Error {
    fn from(err: std::net::AddrParseError) -> Self {
        Error::Internal(err.to_string())
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Error::Internal(err.to_string())
    }
}

impl<T> From<std::sync::PoisonError<T>> for Error {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        Error::Internal(err.to_string())
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(err: tokio::task::JoinError) -> Self {
        Error::Internal(err.to_string())
    }
}

impl From<tokio::sync::mpsc::error::TryRecvError> for Error {
    fn from(err: tokio::sync::mpsc::error::TryRecvError) -> Self {
        Error::Internal(err.to_string())
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for Error {
    fn from(err: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Error::Internal(err.to_string())
    }
}

impl<T> From<tokio::sync::mpsc::error::TrySendError<T>> for Error {
    fn from(err: tokio::sync::mpsc::error::TrySendError<T>) -> Self {
        Error::Internal(err.to_string())
    }
}

impl From<tokio::sync::oneshot::error::RecvError> for Error {
    fn from(err: tokio::sync::oneshot::error::RecvError) -> Self {
        Error::Internal(err.to_string())
    }
}