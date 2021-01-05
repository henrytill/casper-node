use alloc::string::String;
use core::fmt::{Debug, Display};

use base64::DecodeError;
use hex::FromHexError;

use crate::bytesrepr;
use bitflags::_core::fmt::Formatter;
use core::fmt;

/// Cryptographic errors.
#[derive(Debug)]
pub enum Error {
    /// Error resulting from creating or using asymmetric key types.
    AsymmetricKey(String),

    /// Error resulting when decoding a type from a hex-encoded representation.
    FromHex(FromHexError),

    /// Error resulting when serializing or deserializing
    Bytesrepr(bytesrepr::Error),

    /// Error resulting when decoding a type from a base64 representation.
    FromBase64(DecodeError),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, formatter)
    }
}

impl From<FromHexError> for Error {
    fn from(error: FromHexError) -> Self {
        Error::FromHex(error)
    }
}

impl From<bytesrepr::Error> for Error {
    fn from(error: bytesrepr::Error) -> Self {
        Error::Bytesrepr(error)
    }
}
