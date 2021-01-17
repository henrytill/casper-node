//! Asymmetric key types and methods on them

use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use core::{
    cmp::Ordering,
    convert::TryFrom,
    fmt::{self, Debug, Display, Formatter},
    hash::{Hash, Hasher},
    iter,
};

use datasize::DataSize;
use ed25519_dalek::ed25519::signature::Signature as _Signature;
use hex_fmt::HexFmt;
use k256::{self, ecdsa};
#[cfg(feature = "std")]
use schemars::{gen::SchemaGenerator, schema::Schema, JsonSchema};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    account::AccountHash,
    bytesrepr,
    bytesrepr::{FromBytes, ToBytes, U8_SERIALIZED_LENGTH},
    crypto::Error,
    CLType, CLTyped, Tagged,
};

#[cfg(any(feature = "gens", test))]
pub mod gens;
#[cfg(test)]
mod tests;

const TAG_LENGTH: usize = U8_SERIALIZED_LENGTH;

/// Tag for system variant
pub const SYSTEM_TAG: u8 = 0;
const SYSTEM: &str = "System";

/// Tag for ed25519 variant
pub const ED25519_TAG: u8 = 1;
const ED25519: &str = "Ed25519";

/// Tag for secp256k1 variant
pub const SECP256K1_TAG: u8 = 2;
const SECP256K1: &str = "Secp256k1";

const SECP256K1_SECRET_KEY_LENGTH: usize = 32;
const SECP256K1_COMPRESSED_PUBLIC_KEY_LENGTH: usize = 33;
const SECP256K1_SIGNATURE_LENGTH: usize = 64;

/// Public key for system account
pub const SYSTEM_ACCOUNT: PublicKey = PublicKey::System;

/// Operations on asymmetric cryptographic type
pub trait AsymmetricType: Sized + AsRef<[u8]> + Tagged<u8> {
    /// Converts the signature to hex, where the first byte represents the algorithm tag.
    fn to_hex(&self) -> String {
        let bytes = iter::once(&self.tag())
            .chain(self.as_ref())
            .copied()
            .collect::<Vec<u8>>();
        hex::encode(bytes)
    }

    /// Tries to decode a signature from its hex-representation.  The hex format should be as
    /// produced by `Signature::to_hex()`.
    fn from_hex<A: AsRef<[u8]>>(input: A) -> Result<Self, Error> {
        if input.as_ref().len() < 2 {
            return Err(Error::AsymmetricKey("too short".to_string()));
        }

        let (tag_bytes, key_bytes) = input.as_ref().split_at(2);
        let mut tag = [0u8; 1];
        hex::decode_to_slice(tag_bytes, tag.as_mut())?;

        match tag[0] {
            ED25519_TAG => {
                let bytes = hex::decode(key_bytes)?;
                Self::ed25519_from_bytes(&bytes)
            }
            SECP256K1_TAG => {
                let bytes = hex::decode(key_bytes)?;
                Self::secp256k1_from_bytes(&bytes)
            }
            _ => Err(Error::AsymmetricKey(format!(
                "invalid tag.  Expected {} or {}, got {}",
                ED25519_TAG, SECP256K1_TAG, tag[0]
            ))),
        }
    }

    /// Constructs a new system variant.
    fn system() -> Self;

    /// Constructs a new ed25519 variant from a byte slice.
    fn ed25519_from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, Error>;

    /// Constructs a new secp256k1 variant from a byte slice.
    fn secp256k1_from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, Error>;
}

/// A secret or private asymmetric key.
#[derive(DataSize)]
pub enum SecretKey {
    /// System secret key.
    #[data_size(skip)] // Manually verified to have no data on the heap.
    System,
    /// Ed25519 secret key.
    #[data_size(skip)] // Manually verified to have no data on the heap.
    Ed25519(ed25519_dalek::SecretKey),
    /// secp256k1 secret key.
    #[data_size(skip)] // Manually verified to have no data on the heap.
    Secp256k1(k256::SecretKey),
}

impl SecretKey {
    /// The length in bytes of a system secret key.
    pub const SYSTEM_LENGTH: usize = 0;

    /// The length in bytes of an Ed25519 secret key.
    pub const ED25519_LENGTH: usize = ed25519_dalek::SECRET_KEY_LENGTH;

    /// The length in bytes of a secp256k1 secret key.
    pub const SECP256K1_LENGTH: usize = SECP256K1_SECRET_KEY_LENGTH;

    /// Constructs a new Ed25519 variant from a byte array.
    pub fn ed25519(bytes: [u8; Self::ED25519_LENGTH]) -> Self {
        // safe to unwrap as `SecretKey::from_bytes` can only fail if the provided slice is the
        // wrong length.
        SecretKey::Ed25519(ed25519_dalek::SecretKey::from_bytes(&bytes).unwrap())
    }

    /// Constructs a new secp256k1 variant from a byte array.
    pub fn secp256k1(bytes: [u8; Self::SECP256K1_LENGTH]) -> Self {
        // safe to unwrap as `SecretKey::from_bytes` can only fail if the provided slice is the
        // wrong length.
        SecretKey::Secp256k1(k256::SecretKey::from_bytes(&bytes).unwrap())
    }

    /// Exposes the secret values of the key as a byte slice.
    pub fn as_slice(&self) -> &[u8] {
        match self {
            SecretKey::System => &[],
            SecretKey::Ed25519(secret_key) => secret_key.as_ref(),
            SecretKey::Secp256k1(secret_key) => secret_key.as_bytes().as_slice(),
        }
    }

    fn variant_name(&self) -> &str {
        match self {
            SecretKey::System => SYSTEM,
            SecretKey::Ed25519(_) => ED25519,
            SecretKey::Secp256k1(_) => SECP256K1,
        }
    }
}

impl AsymmetricType for SecretKey {
    fn system() -> Self {
        SecretKey::System
    }

    fn ed25519_from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, Error> {
        Ok(SecretKey::Ed25519(
            ed25519_dalek::SecretKey::from_bytes(bytes.as_ref()).map_err(|_| {
                Error::AsymmetricKey(format!(
                    "failed to construct Ed25519 secret key.  Expected {} bytes, got {} bytes.",
                    Self::ED25519_LENGTH,
                    bytes.as_ref().len()
                ))
            })?,
        ))
    }

    fn secp256k1_from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, Error> {
        Ok(SecretKey::Secp256k1(
            k256::SecretKey::from_bytes(bytes.as_ref()).map_err(|_| {
                Error::AsymmetricKey(format!(
                    "failed to construct secp256k1 secret key.  Expected {} bytes, got {} bytes.",
                    Self::SECP256K1_LENGTH,
                    bytes.as_ref().len()
                ))
            })?,
        ))
    }
}

impl AsRef<[u8]> for SecretKey {
    fn as_ref(&self) -> &[u8] {
        match self {
            SecretKey::System => &[],
            SecretKey::Ed25519(secret_key) => secret_key.as_ref(),
            SecretKey::Secp256k1(secret_key) => secret_key.as_bytes(),
        }
    }
}

impl Clone for SecretKey {
    fn clone(&self) -> Self {
        match self {
            SecretKey::System => SecretKey::System,
            SecretKey::Ed25519(sk) => Self::ed25519_from_bytes(sk.as_ref()).unwrap(),
            SecretKey::Secp256k1(sk) => Self::secp256k1_from_bytes(sk.as_bytes()).unwrap(),
        }
    }
}

impl Debug for SecretKey {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "SecretKey::{}({})",
            self.variant_name(),
            HexFmt(self)
        )
    }
}

impl Display for SecretKey {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "SecKey::{}({:10})",
            self.variant_name(),
            HexFmt(self)
        )
    }
}

impl Tagged<u8> for SecretKey {
    fn tag(&self) -> u8 {
        match self {
            SecretKey::System => SYSTEM_TAG,
            SecretKey::Ed25519(_) => ED25519_TAG,
            SecretKey::Secp256k1(_) => SECP256K1_TAG,
        }
    }
}

impl ToBytes for SecretKey {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        let mut buffer = bytesrepr::allocate_buffer(self)?;
        match self {
            SecretKey::System => buffer.insert(0, SYSTEM_TAG),
            SecretKey::Ed25519(public_key) => {
                buffer.insert(0, ED25519_TAG);
                let ed25519_bytes = public_key.as_bytes();
                buffer.extend_from_slice(ed25519_bytes);
            }
            SecretKey::Secp256k1(public_key) => {
                buffer.insert(0, SECP256K1_TAG);
                let secp256k1_bytes = public_key.as_bytes();
                buffer.extend_from_slice(secp256k1_bytes);
            }
        }
        Ok(buffer)
    }

    fn serialized_length(&self) -> usize {
        TAG_LENGTH
            + match self {
                SecretKey::System => Self::SYSTEM_LENGTH,
                SecretKey::Ed25519(_) => Self::ED25519_LENGTH,
                SecretKey::Secp256k1(_) => Self::SECP256K1_LENGTH,
            }
    }
}

impl FromBytes for SecretKey {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (tag, remainder) = u8::from_bytes(bytes)?;
        match tag {
            SYSTEM_TAG => Ok((SecretKey::System, remainder)),
            ED25519_TAG => {
                let (raw_bytes, remainder): ([u8; Self::ED25519_LENGTH], _) =
                    FromBytes::from_bytes(remainder)?;
                let secret_key = Self::ed25519(raw_bytes);
                Ok((secret_key, remainder))
            }
            SECP256K1_TAG => {
                let (raw_bytes, remainder): ([u8; Self::SECP256K1_LENGTH], _) =
                    FromBytes::from_bytes(remainder)?;
                let secret_key = Self::secp256k1(raw_bytes);
                Ok((secret_key, remainder))
            }
            _ => Err(bytesrepr::Error::Formatting),
        }
    }
}

impl Serialize for SecretKey {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        detail::serialize(self, serializer)
    }
}

impl<'de> Deserialize<'de> for SecretKey {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        detail::deserialize(deserializer)
    }
}

/// A public asymmetric key.
#[derive(Clone, Copy, DataSize, Eq, PartialEq)]
pub enum PublicKey {
    /// System public key.
    #[data_size(skip)] // Manually verified to have no data on the heap.
    System,
    /// Ed25519 public key.
    #[data_size(skip)] // Manually verified to have no data on the heap.
    Ed25519(ed25519_dalek::PublicKey),
    /// secp256k1 public key.
    #[data_size(skip)] // Manually verified to have no data on the heap.
    Secp256k1(k256::PublicKey),
}

impl PublicKey {
    /// The length in bytes of a system public key.
    pub const SYSTEM_LENGTH: usize = 0;

    /// The length in bytes of an Ed25519 public key.
    pub const ED25519_LENGTH: usize = ed25519_dalek::PUBLIC_KEY_LENGTH;

    /// The length in bytes of a secp256k1 public key.
    pub const SECP256K1_LENGTH: usize = SECP256K1_COMPRESSED_PUBLIC_KEY_LENGTH;

    /// Constructs a new Ed25519 variant from a byte array.
    pub fn ed25519(bytes: [u8; Self::ED25519_LENGTH]) -> Result<Self, Error> {
        Ok(PublicKey::Ed25519(
            ed25519_dalek::PublicKey::from_bytes(&bytes).map_err(|_| {
                Error::AsymmetricKey(format!(
                    "failed to construct Ed25519 public key from {:?}",
                    bytes
                ))
            })?,
        ))
    }

    /// Constructs a new secp256k1 variant from a byte array.
    pub fn secp256k1(bytes: [u8; Self::SECP256K1_LENGTH]) -> Result<Self, Error> {
        Ok(PublicKey::Secp256k1(
            k256::PublicKey::from_bytes(&bytes[..]).ok_or_else(|| {
                Error::AsymmetricKey(format!(
                    "failed to construct secp256k1 public key from {:?}",
                    &bytes[..]
                ))
            })?,
        ))
    }

    /// Creates an `AccountHash` from a given `PublicKey` instance.
    pub fn to_account_hash(&self) -> AccountHash {
        AccountHash::from(self)
    }

    /// Returns a human-readable version of `self`, with the inner bytes encoded to Base16.
    pub fn to_formatted_string(&self) -> String {
        self.to_hex()
    }

    /// Parses a string formatted as per `Self::to_formatted_string()` into a [`PublicKey`].
    pub fn from_formatted_str(str: &str) -> Result<Self, Error> {
        Self::from_hex(str)
    }

    fn variant_name(&self) -> &str {
        match self {
            PublicKey::System => SYSTEM,
            PublicKey::Ed25519(_) => ED25519,
            PublicKey::Secp256k1(_) => SECP256K1,
        }
    }
}

impl AsymmetricType for PublicKey {
    fn system() -> Self {
        PublicKey::System
    }

    fn ed25519_from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, Error> {
        Ok(PublicKey::Ed25519(
            ed25519_dalek::PublicKey::from_bytes(bytes.as_ref()).map_err(|_| {
                Error::AsymmetricKey(format!(
                    "failed to construct Ed25519 public key.  Expected {} bytes, got {} bytes.",
                    Self::ED25519_LENGTH,
                    bytes.as_ref().len()
                ))
            })?,
        ))
    }

    fn secp256k1_from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, Error> {
        let mut public_key = k256::PublicKey::from_bytes(bytes.as_ref()).ok_or_else(|| {
            Error::AsymmetricKey(format!(
                "failed to construct secp256k1 public key.  Expected {} bytes, got {} bytes.",
                Self::SECP256K1_LENGTH,
                bytes.as_ref().len()
            ))
        })?;
        public_key.compress();
        Ok(PublicKey::Secp256k1(public_key))
    }
}

impl From<&SecretKey> for PublicKey {
    fn from(secret_key: &SecretKey) -> PublicKey {
        match secret_key {
            SecretKey::System => PublicKey::System,
            SecretKey::Ed25519(secret_key) => PublicKey::Ed25519(secret_key.into()),
            SecretKey::Secp256k1(secret_key) => PublicKey::Secp256k1(
                k256::PublicKey::from_secret_key(secret_key, true)
                    .expect("should create secp256k1 public key"),
            ),
        }
    }
}

impl From<SecretKey> for PublicKey {
    fn from(secret_key: SecretKey) -> PublicKey {
        PublicKey::from(&secret_key)
    }
}

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        match self {
            PublicKey::System => &[],
            PublicKey::Ed25519(public_key) => public_key.as_ref(),
            PublicKey::Secp256k1(public_key) => public_key.as_ref(),
        }
    }
}

impl Debug for PublicKey {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "PublicKey::{}({})",
            self.variant_name(),
            HexFmt(self)
        )
    }
}

impl Display for PublicKey {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "PubKey::{}({:10})",
            self.variant_name(),
            HexFmt(self)
        )
    }
}

impl PartialOrd for PublicKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PublicKey {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_tag = self.tag();
        let other_tag = other.tag();
        if self_tag == other_tag {
            self.as_ref().cmp(other.as_ref())
        } else {
            self_tag.cmp(&other_tag)
        }
    }
}

// This implementation of `Hash` agrees with the derived `PartialEq`.  It's required since
// `ed25519_dalek::PublicKey` doesn't implement `Hash`.
#[allow(clippy::derive_hash_xor_eq)]
impl Hash for PublicKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tag().hash(state);
        self.as_ref().hash(state);
    }
}

impl Tagged<u8> for PublicKey {
    fn tag(&self) -> u8 {
        match self {
            PublicKey::System => SYSTEM_TAG,
            PublicKey::Ed25519(_) => ED25519_TAG,
            PublicKey::Secp256k1(_) => SECP256K1_TAG,
        }
    }
}

impl ToBytes for PublicKey {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        let mut buffer = bytesrepr::allocate_buffer(self)?;
        match self {
            PublicKey::System => {
                buffer.insert(0, SYSTEM_TAG);
            }
            PublicKey::Ed25519(public_key) => {
                buffer.insert(0, ED25519_TAG);
                let ed25519_bytes = public_key.as_bytes();
                buffer.extend_from_slice(ed25519_bytes);
            }
            PublicKey::Secp256k1(public_key) => {
                buffer.insert(0, SECP256K1_TAG);
                let secp256k1_bytes = public_key.as_bytes();
                buffer.extend_from_slice(secp256k1_bytes);
            }
        }
        Ok(buffer)
    }

    fn serialized_length(&self) -> usize {
        TAG_LENGTH
            + match self {
                PublicKey::System => Self::SYSTEM_LENGTH,
                PublicKey::Ed25519(_) => Self::ED25519_LENGTH,
                PublicKey::Secp256k1(_) => Self::SECP256K1_LENGTH,
            }
    }
}

impl FromBytes for PublicKey {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (tag, remainder) = u8::from_bytes(bytes)?;
        match tag {
            SYSTEM_TAG => Ok((PublicKey::System, remainder)),
            ED25519_TAG => {
                let (raw_bytes, remainder): ([u8; Self::ED25519_LENGTH], _) =
                    FromBytes::from_bytes(remainder)?;
                let public_key =
                    Self::ed25519(raw_bytes).map_err(|_error| bytesrepr::Error::Formatting)?;
                Ok((public_key, remainder))
            }
            SECP256K1_TAG => {
                let (raw_bytes, remainder): ([u8; Self::SECP256K1_LENGTH], _) =
                    FromBytes::from_bytes(remainder)?;
                let public_key =
                    Self::secp256k1(raw_bytes).map_err(|_error| bytesrepr::Error::Formatting)?;
                Ok((public_key, remainder))
            }
            _ => Err(bytesrepr::Error::Formatting),
        }
    }
}

impl Serialize for PublicKey {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        detail::serialize(self, serializer)
    }
}

impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        detail::deserialize(deserializer)
    }
}

#[cfg(feature = "std")]
impl JsonSchema for PublicKey {
    fn schema_name() -> String {
        String::from("PublicKey")
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let schema = gen.subschema_for::<String>();
        let mut schema_object = schema.into_object();
        schema_object.metadata().description = Some(
            "Hex-encoded cryptographic public key, including the algorithm tag prefix.".to_string(),
        );
        schema_object.into()
    }
}

impl CLTyped for PublicKey {
    fn cl_type() -> CLType {
        CLType::PublicKey
    }
}

/// A signature of given data.
#[derive(Clone, Copy, DataSize)]
pub enum Signature {
    /// System signature.  Cannot be verified.
    System,
    /// Ed25519 signature.
    //
    // This is held as a byte array rather than an `ed25519_dalek::Signature` as that type doesn't
    // implement `AsRef` amongst other common traits.  In order to implement these common traits,
    // it is convenient and cheap to use `signature.as_ref()`.
    Ed25519([u8; ed25519_dalek::SIGNATURE_LENGTH]),
    /// secp256k1 signature.
    #[data_size(skip)] // Manually verified to have no data on the heap.
    Secp256k1(ecdsa::Signature),
}

impl Signature {
    /// The length in bytes of a system signature,
    pub const SYSTEM_LENGTH: usize = 0;

    /// The length in bytes of an Ed25519 signature,
    pub const ED25519_LENGTH: usize = ed25519_dalek::SIGNATURE_LENGTH;

    /// The length in bytes of a secp256k1 signature
    pub const SECP256K1_LENGTH: usize = SECP256K1_SIGNATURE_LENGTH;

    /// Constructs a new Ed25519 variant from a byte array.
    pub fn ed25519(bytes: [u8; Self::ED25519_LENGTH]) -> Result<Self, Error> {
        let signature = ed25519_dalek::Signature::from_bytes(&bytes).map_err(|_| {
            Error::AsymmetricKey(format!(
                "failed to construct Ed25519 signature from {:?}",
                &bytes[..]
            ))
        })?;

        Ok(Signature::Ed25519(signature.to_bytes()))
    }

    /// Constructs a new secp256k1 variant from a byte array.
    pub fn secp256k1(bytes: [u8; Self::SECP256K1_LENGTH]) -> Result<Self, Error> {
        let signature = ecdsa::Signature::try_from(&bytes[..]).map_err(|_| {
            Error::AsymmetricKey(format!(
                "failed to construct secp256k1 signature from {:?}",
                &bytes[..]
            ))
        })?;

        Ok(Signature::Secp256k1(signature))
    }

    fn variant_name(&self) -> &str {
        match self {
            Signature::System => SYSTEM,
            Signature::Ed25519(_) => ED25519,
            Signature::Secp256k1(_) => SECP256K1,
        }
    }
}

impl AsymmetricType for Signature {
    fn system() -> Self {
        Signature::System
    }

    fn ed25519_from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, Error> {
        let signature = ed25519_dalek::Signature::from_bytes(bytes.as_ref()).map_err(|_| {
            Error::AsymmetricKey(format!(
                "failed to construct Ed25519 signature from {:?}",
                bytes.as_ref()
            ))
        })?;
        Ok(Signature::Ed25519(signature.to_bytes()))
    }

    fn secp256k1_from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, Error> {
        let signature = ecdsa::Signature::try_from(bytes.as_ref()).map_err(|_| {
            Error::AsymmetricKey(format!(
                "failed to construct secp256k1 signature from {:?}",
                bytes.as_ref()
            ))
        })?;
        Ok(Signature::Secp256k1(signature))
    }
}

impl AsRef<[u8]> for Signature {
    fn as_ref(&self) -> &[u8] {
        match self {
            Signature::System => &[],
            Signature::Ed25519(signature) => signature.as_ref(),
            Signature::Secp256k1(signature) => signature.as_ref(),
        }
    }
}

impl Debug for Signature {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Signature::{}({})",
            self.variant_name(),
            HexFmt(self)
        )
    }
}

impl Display for Signature {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Sig::{}({:10})",
            self.variant_name(),
            HexFmt(self)
        )
    }
}

impl PartialOrd for Signature {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Signature {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_tag = self.tag();
        let other_tag = other.tag();
        if self_tag == other_tag {
            self.as_ref().cmp(other.as_ref())
        } else {
            self_tag.cmp(&other_tag)
        }
    }
}

impl PartialEq for Signature {
    fn eq(&self, other: &Self) -> bool {
        self.tag() == other.tag() && self.as_ref() == other.as_ref()
    }
}

impl Eq for Signature {}

impl Hash for Signature {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tag().hash(state);
        self.as_ref().hash(state);
    }
}

impl Tagged<u8> for Signature {
    fn tag(&self) -> u8 {
        match self {
            Signature::System => SYSTEM_TAG,
            Signature::Ed25519(_) => ED25519_TAG,
            Signature::Secp256k1(_) => SECP256K1_TAG,
        }
    }
}

impl ToBytes for Signature {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        let mut buffer = bytesrepr::allocate_buffer(self)?;
        match self {
            Signature::System => {
                buffer.insert(0, SYSTEM_TAG);
            }
            Signature::Ed25519(signature) => {
                buffer.insert(0, ED25519_TAG);
                let ed5519_bytes = signature.to_bytes()?;
                buffer.extend(ed5519_bytes);
            }
            Signature::Secp256k1(signature) => {
                buffer.insert(0, SECP256K1_TAG);
                let secp256k1_bytes = signature.as_ref();
                buffer.extend_from_slice(secp256k1_bytes);
            }
        }
        Ok(buffer)
    }

    fn serialized_length(&self) -> usize {
        TAG_LENGTH
            + match self {
                Signature::System => Self::SYSTEM_LENGTH,
                Signature::Ed25519(_) => Self::ED25519_LENGTH,
                Signature::Secp256k1(_) => Self::SECP256K1_LENGTH,
            }
    }
}

impl FromBytes for Signature {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (tag, remainder) = u8::from_bytes(bytes)?;
        match tag {
            SYSTEM_TAG => Ok((Signature::System, remainder)),
            ED25519_TAG => {
                let (raw_bytes, remainder): ([u8; Self::ED25519_LENGTH], _) =
                    FromBytes::from_bytes(remainder)?;
                let public_key =
                    Self::ed25519(raw_bytes).map_err(|_error| bytesrepr::Error::Formatting)?;
                Ok((public_key, remainder))
            }
            SECP256K1_TAG => {
                let (raw_bytes, remainder): ([u8; Self::SECP256K1_LENGTH], _) =
                    FromBytes::from_bytes(remainder)?;
                let public_key =
                    Self::secp256k1(raw_bytes).map_err(|_error| bytesrepr::Error::Formatting)?;
                Ok((public_key, remainder))
            }
            _ => Err(bytesrepr::Error::Formatting),
        }
    }
}

impl Serialize for Signature {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        detail::serialize(self, serializer)
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        detail::deserialize(deserializer)
    }
}

#[cfg(feature = "std")]
impl JsonSchema for Signature {
    fn schema_name() -> String {
        String::from("Signature")
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let schema = gen.subschema_for::<String>();
        let mut schema_object = schema.into_object();
        schema_object.metadata().description = Some(
            "Hex-encoded cryptographic signature, including the algorithm tag prefix.".to_string(),
        );
        schema_object.into()
    }
}

mod detail {
    use alloc::string::String;

    use serde::{de::Error as _deError, Deserialize, Deserializer, Serialize, Serializer};

    use super::{PublicKey, SecretKey, Signature};
    use crate::AsymmetricType;

    /// Used to serialize and deserialize asymmetric key types where the (de)serializer is not a
    /// human-readable type.
    ///
    /// The wrapped contents are the result of calling `t_as_ref()` on the type.
    #[derive(Serialize, Deserialize)]
    pub enum AsymmetricTypeAsBytes<'a> {
        System,
        Ed25519(&'a [u8]),
        Secp256k1(&'a [u8]),
    }

    impl<'a> From<&'a SecretKey> for AsymmetricTypeAsBytes<'a> {
        fn from(secret_key: &'a SecretKey) -> Self {
            match secret_key {
                SecretKey::System => AsymmetricTypeAsBytes::System,
                SecretKey::Ed25519(ed25519) => AsymmetricTypeAsBytes::Ed25519(ed25519.as_ref()),
                SecretKey::Secp256k1(secp256k1) => {
                    AsymmetricTypeAsBytes::Secp256k1(secp256k1.as_bytes().as_slice())
                }
            }
        }
    }

    impl<'a> From<&'a PublicKey> for AsymmetricTypeAsBytes<'a> {
        fn from(public_key: &'a PublicKey) -> Self {
            match public_key {
                PublicKey::System => AsymmetricTypeAsBytes::System,
                PublicKey::Ed25519(ed25519) => AsymmetricTypeAsBytes::Ed25519(ed25519.as_ref()),
                PublicKey::Secp256k1(secp256k1) => {
                    AsymmetricTypeAsBytes::Secp256k1(secp256k1.as_ref())
                }
            }
        }
    }

    impl<'a> From<&'a Signature> for AsymmetricTypeAsBytes<'a> {
        fn from(signature: &'a Signature) -> Self {
            match signature {
                Signature::System => AsymmetricTypeAsBytes::System,
                Signature::Ed25519(ed25519) => AsymmetricTypeAsBytes::Ed25519(ed25519.as_ref()),
                Signature::Secp256k1(secp256k1) => {
                    AsymmetricTypeAsBytes::Secp256k1(secp256k1.as_ref())
                }
            }
        }
    }

    pub fn serialize<'a, T, S>(value: &'a T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: AsymmetricType,
        S: Serializer,
        AsymmetricTypeAsBytes<'a>: From<&'a T>,
    {
        if serializer.is_human_readable() {
            return value.to_hex().serialize(serializer);
        }

        AsymmetricTypeAsBytes::from(value).serialize(serializer)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: AsymmetricType,
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let hex_string = String::deserialize(deserializer)?;
            let value = T::from_hex(hex_string.as_bytes()).map_err(D::Error::custom)?;
            return Ok(value);
        }

        let as_bytes = AsymmetricTypeAsBytes::deserialize(deserializer)?;
        match as_bytes {
            AsymmetricTypeAsBytes::System => Ok(T::system()),
            AsymmetricTypeAsBytes::Ed25519(raw_bytes) => {
                T::ed25519_from_bytes(raw_bytes).map_err(D::Error::custom)
            }
            AsymmetricTypeAsBytes::Secp256k1(raw_bytes) => {
                T::secp256k1_from_bytes(raw_bytes).map_err(D::Error::custom)
            }
        }
    }
}
