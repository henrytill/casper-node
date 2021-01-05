//! TODO

mod asymmetric_key;
mod common;
mod error;
mod tagged;

#[cfg(any(feature = "gens", test))]
pub use asymmetric_key::gens;
pub use asymmetric_key::{PublicKey, SecretKey, Signature, ED25519_TAG, SECP256K1_TAG};
pub use error::Error;
pub use tagged::Tagged;
