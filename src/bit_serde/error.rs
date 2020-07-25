use serde::{de, ser};
use std::fmt::{self, Display, Formatter};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    Message(String),

    Eof,
    Unsupported(String),
    TrailingCharacters,
    NumberOverflow,
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::Message(msg) => f.write_str(msg),
            Error::Eof => f.write_str("unexpected end of input"),
            Error::Unsupported(type_) => f.write_str(&format!("{} is unsupported", type_)),
            Error::TrailingCharacters => f.write_str("not all bits were consumed"),
            Error::NumberOverflow => f.write_str("number is too big or too big on the negative"),
        }
    }
}

impl std::error::Error for Error {}
