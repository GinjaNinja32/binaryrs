use std::num::TryFromIntError;
use std::string::FromUtf8Error;

pub type Result<T> = std::result::Result<T, BinError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinError {
    // There was insufficient data to successfully deserialize the data.
    // The parameter indicates how much data is required in order to make further progress (i.e. to finish decoding
    // the field that required more data). It may be zero to denote that the amount of data required is unknown.
    InsufficientData(usize),
    IntTooLarge(TryFromIntError),
    InvalidUTF8(FromUtf8Error),
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
