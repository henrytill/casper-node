use std::convert::TryFrom;

use casper_types::PublicKey;

use crate::engine_server::{mappings::ParsingError, state};

impl From<PublicKey> for state::PublicKey {
    fn from(_public_key: PublicKey) -> Self {
        unimplemented!()
    }
}

impl TryFrom<state::PublicKey> for PublicKey {
    type Error = ParsingError;

    fn try_from(_pb_public_key: state::PublicKey) -> Result<Self, Self::Error> {
        unimplemented!()
    }
}
