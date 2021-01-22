use std::num::TryFromIntError;
use std::string::FromUtf8Error;

pub type Result<T> = std::result::Result<T, BinError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinError {
    // There was insufficient data to successfully deserialize the type.
    InsufficientData,
    // A variant tag was parsed that did not correspond to a known enum variant.
    // The parameter indicates the invalid variant tag.
    VariantNotMatched(u64),
    IntTooLarge(TryFromIntError),
    InvalidUTF8(FromUtf8Error),
    IOError(String),

    Custom(String),
}

impl std::error::Error for BinError {}
impl std::fmt::Display for BinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<TryFromIntError> for BinError {
    fn from(other: TryFromIntError) -> Self {
        Self::IntTooLarge(other)
    }
}

impl From<FromUtf8Error> for BinError {
    fn from(other: FromUtf8Error) -> Self {
        Self::InvalidUTF8(other)
    }
}

impl From<std::io::Error> for BinError {
    fn from(other: std::io::Error) -> Self {
        let s = format!("{}", other);
        if s == "failed to fill whole buffer" {
            Self::InsufficientData
        } else {
            Self::IOError(s)
        }
    }
}
