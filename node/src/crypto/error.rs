use std::result;

use base64::DecodeError;
use hex::FromHexError;
use pem::PemError;
use thiserror::Error;

use crate::utils::{ReadFileError, WriteFileError};
use casper_types::crypto;

/// A specialized `std::result::Result` type for cryptographic errors.
pub type Result<T> = result::Result<T, Error>;

/// Cryptographic errors.
#[derive(Debug, Error)]
pub enum Error {
    /// Error resulting from creating or using asymmetric key types.
    #[error("asymmetric key error: {0}")]
    AsymmetricKey(String),

    /// Error resulting when decoding a type from a hex-encoded representation.
    #[error("parsing from hex: {0}")]
    FromHex(#[from] FromHexError),

    /// Error trying to read a secret key.
    #[error("secret key load failed: {0}")]
    SecretKeyLoad(ReadFileError),

    /// Error trying to read a public key.
    #[error("public key load failed: {0}")]
    PublicKeyLoad(ReadFileError),

    /// Error resulting when decoding a type from a base64 representation.
    #[error("decoding error: {0}")]
    FromBase64(#[from] DecodeError),

    /// Pem format error.
    #[error("pem error: {0}")]
    FromPem(String),

    /// DER format error.
    #[error("der error: {0}")]
    FromDer(#[from] derp::Error),

    /// Error trying to write a secret key.
    #[error("secret key save failed: {0}")]
    SecretKeySave(WriteFileError),

    /// Error trying to write a public key.
    #[error("public key save failed: {0}")]
    PublicKeySave(WriteFileError),
}

impl From<PemError> for Error {
    fn from(error: PemError) -> Self {
        Error::FromPem(error.to_string())
    }
}

impl From<crypto::Error> for Error {
    fn from(error: crypto::Error) -> Self {
        match error {
            crypto::Error::AsymmetricKey(string) => Error::AsymmetricKey(string),
            crypto::Error::FromHex(error) => Error::FromHex(error),
            crypto::Error::Bytesrepr(_) => unimplemented!(),
            crypto::Error::FromBase64(error) => Error::FromBase64(error),
        }
    }
}
