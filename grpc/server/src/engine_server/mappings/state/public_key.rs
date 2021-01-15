use std::convert::TryFrom;

use casper_types::{bytesrepr, PublicKey};

use crate::engine_server::{mappings::ParsingError, state};
use casper_types::bytesrepr::ToBytes;

impl From<PublicKey> for state::PublicKey {
    fn from(public_key: PublicKey) -> Self {
        let mut pb_public_key = state::PublicKey::new();
        pb_public_key.set_public_key(public_key.to_bytes().unwrap());
        pb_public_key
    }
}

impl TryFrom<state::PublicKey> for PublicKey {
    type Error = ParsingError;

    fn try_from(pb_public_key: state::PublicKey) -> Result<Self, Self::Error> {
        let bytes = pb_public_key.get_public_key().to_vec();
        bytesrepr::deserialize(bytes).map_err(|_| ParsingError(String::from("PublicKey")))
    }
}
