use alloc::string::String;

use crate::{
    bytesrepr::{self, FromBytes, ToBytes},
    crypto::Error,
};

pub fn to_hex<T: ToBytes>(value: &T) -> String {
    let bytes = value.to_bytes().expect("should allocate");
    hex::encode(bytes)
}

pub fn from_hex<A: AsRef<[u8]>, B: FromBytes>(input: A) -> Result<B, Error> {
    let bytes = hex::decode(input)?;
    bytesrepr::deserialize(bytes).map_err(Into::into)
}
