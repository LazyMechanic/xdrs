use std::{
    error::Error as StdError,
    fmt::{Debug, Display, Formatter},
};

pub struct Error {
    err: Box<ErrorImpl>,
}

impl Error {
    pub fn at_field(mut self, field: &'static str) -> Self {
        self.err.path.push(field);
        self
    }

    pub fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::from(ErrorCode::Message(msg.to_string()))
    }

    pub fn io(err: std::io::Error) -> Self {
        Self::from(ErrorCode::Io(err))
    }

    pub fn invalid_value<U, E>(actual: U, expected: E) -> Self
    where
        U: Display,
        E: Display,
    {
        Self::from(ErrorCode::InvalidValue(format!(
            "invalid value: {actual}, expected: {expected}"
        )))
    }

    pub fn invalid_size(sz: usize, expsz: ExpectedSize) -> Self {
        Self::from(ErrorCode::InvalidSize {
            actual: sz,
            expected: expsz,
        })
    }

    pub fn unknown_variant(
        variant: i32,
        expected: &'static [ExpectedVariant],
    ) -> Self {
        Self::from(ErrorCode::UnknownVariant {
            actual: variant,
            expected,
        })
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.err.path.is_empty() {
            write!(f, "Error(\"{}\")", self.err.code)
        } else {
            write!(
                f,
                "Error(\"{}\", path: \"{}\")",
                self.err.code, self.err.path,
            )
        }
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
            err: Box::new(ErrorImpl {
                code: value,
                path: Default::default(),
            }),
        }
    }
}

struct ErrorImpl {
    code: ErrorCode,
    path: ErrorPath,
}

impl Display for ErrorImpl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.path.is_empty() {
            Display::fmt(&self.code, f)
        } else {
            write!(f, "{} at `{}`", self.code, self.path)
        }
    }
}

#[derive(Debug, Default)]
struct ErrorPath {
    path: Vec<&'static str>,
}

impl ErrorPath {
    pub fn push(&mut self, path: &'static str) {
        self.path.push(path);
    }

    pub fn is_empty(&self) -> bool {
        self.path.is_empty()
    }
}

impl Display for ErrorPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for p in &self.path {
            write!(f, ".{p}")?;
        }
        Ok(())
    }
}

pub(crate) enum ErrorCode {
    Message(String),
    Io(std::io::Error),
    InvalidValue(String),
    InvalidSize {
        actual: usize,
        expected: ExpectedSize,
    },
    UnknownVariant {
        actual: i32,
        expected: &'static [ExpectedVariant],
    },
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCode::Message(msg) => f.write_str(msg),
            ErrorCode::Io(err) => Display::fmt(err, f),
            ErrorCode::InvalidValue(msg) => f.write_str(msg),
            ErrorCode::InvalidSize { actual, expected } => {
                write!(
                    f,
                    "invalid size: {actual}, expected: {}",
                    expected.display()
                )
            }
            ErrorCode::UnknownVariant { actual, expected } => {
                if expected.is_empty() {
                    write!(f, "unknown variant `{actual}`")
                } else {
                    write!(
                        f,
                        "unknown variant `{actual}`, expected {}",
                        OneOfVariant { variants: expected }
                    )
                }
            }
        }
    }
}

/// Used in error messages.
///
/// - expected 1
/// - expected 1 or 2
/// - expected one of 1..=3, 5, 6
struct OneOfVariant {
    variants: &'static [ExpectedVariant],
}

impl Display for OneOfVariant {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.variants.len() {
            0 => Ok(()), // special case elsewhere
            1 => write!(f, "{}", self.variants[0].display()),
            2 => {
                write!(
                    f,
                    "{} or {}",
                    self.variants[0].display(),
                    self.variants[1].display()
                )
            }
            _ => {
                f.write_str("one of ")?;
                for (i, elem) in self.variants.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{}", elem.display())?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ExpectedSize {
    Exact(usize),
    Max(usize),
}

impl ExpectedSize {
    fn display(&self) -> impl Display {
        struct ExpectedSizeDisplay {
            expsz: ExpectedSize,
        }

        impl Display for ExpectedSizeDisplay {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                match self.expsz {
                    ExpectedSize::Exact(expsz) => {
                        write!(f, "actual size must be exactly {expsz}")
                    }
                    ExpectedSize::Max(expsz) => {
                        write!(f, "actual size must be at most {expsz}")
                    }
                }
            }
        }

        ExpectedSizeDisplay { expsz: *self }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpectedVariant {
    Value(i32),
    Range(std::ops::RangeInclusive<i32>),
}

impl ExpectedVariant {
    fn display(&self) -> impl Display + '_ {
        struct ExpectedVariantDisplay<'a>(&'a ExpectedVariant);
        impl Display for ExpectedVariantDisplay<'_> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                match self.0 {
                    ExpectedVariant::Value(v) => write!(f, "{}", v)?,
                    ExpectedVariant::Range(r) => {
                        Display::fmt(r.start(), f)?;
                        write!(f, "..=")?;
                        Display::fmt(r.end(), f)?;
                    }
                }
                Ok(())
            }
        }

        ExpectedVariantDisplay(self)
    }
}

pub trait ResultExt<T> {
    fn at_field(self, field: &'static str) -> Result<T, Error>;
}

impl<T> ResultExt<T> for Result<T, Error> {
    fn at_field(self, field: &'static str) -> Result<T, Error> {
        match self {
            Ok(v) => Ok(v),
            Err(err) => Err(err.at_field(field)),
        }
    }
}
