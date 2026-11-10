use std::{
    error::Error as StdError,
    fmt::{Debug, Display, Formatter},
};

use crate::{DecodeError, EncodeError, ExpectedSize};

pub struct Error {
    err: Box<ErrorImpl>,
}

impl Error {
    pub(crate) fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::from(ErrorCode::Custom(msg.to_string()))
    }

    pub(crate) fn io(err: std::io::Error) -> Self {
        Self::from(ErrorCode::Io(err))
    }

    pub(crate) fn invalid_size(len: usize, explen: ExpectedSize) -> Self {
        Self::from(ErrorCode::InvalidSize {
            actual: len,
            expected: explen,
        })
    }

    pub(crate) fn unknown_variant(variant: i32) -> Self {
        Self::from(ErrorCode::UnknownVariant { variant })
    }

    pub(crate) fn bad_utf8() -> Self {
        Self::from(ErrorCode::BadUtf8)
    }
}

impl DecodeError for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Self::custom(msg)
    }

    fn invalid_size(len: usize, explen: ExpectedSize) -> Self {
        Self::invalid_size(len, explen)
    }
}

impl EncodeError for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Self::custom(msg)
    }

    fn invalid_size(len: usize, explen: ExpectedSize) -> Self {
        Self::invalid_size(len, explen)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error(\"{}\")", self.err.code)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        Display::fmt(&*self.err, f)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match &self.err.code {
            ErrorCode::Io(err) => err.source(),
            _ => None,
        }
    }
}

impl From<ErrorCode> for Error {
    fn from(value: ErrorCode) -> Self {
        Self {
            err: Box::new(ErrorImpl { code: value }),
        }
    }
}

struct ErrorImpl {
    code: ErrorCode,
}

impl Display for ErrorImpl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.code, f)
    }
}

pub(crate) enum ErrorCode {
    Custom(String),
    Io(std::io::Error),
    InvalidSize {
        actual: usize,
        expected: ExpectedSize,
    },
    UnknownVariant {
        variant: i32,
    },
    BadUtf8,
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCode::Custom(msg) => f.write_str(msg),
            ErrorCode::Io(err) => Display::fmt(err, f),
            ErrorCode::InvalidSize { actual, expected } => {
                write!(
                    f,
                    "invalid size: {actual}, expected: {}",
                    expected.display()
                )
            }
            ErrorCode::UnknownVariant { variant } => {
                write!(f, "unknown variant: `{variant}`")
            }
            ErrorCode::BadUtf8 => {
                f.write_str("decoded string was not valid UTF-8")
            }
        }
    }
}
