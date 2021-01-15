use std::collections::BTreeSet;

use casper_types::{DeployHash, PublicKey};

use crate::core::engine_state::executable_deploy_item::ExecutableDeployItem;

type GasPrice = u64;

/// Represents a deploy to be executed.  Corresponds to the similarly-named ipc protobuf message.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DeployItem {
    pub public_key: PublicKey,
    pub session: ExecutableDeployItem,
    pub payment: ExecutableDeployItem,
    pub gas_price: GasPrice,
    pub authorization_keys: BTreeSet<PublicKey>,
    pub deploy_hash: DeployHash,
}

impl DeployItem {
    /// Creates a [`DeployItem`].
    pub fn new(
        public_key: PublicKey,
        session: ExecutableDeployItem,
        payment: ExecutableDeployItem,
        gas_price: GasPrice,
        authorization_keys: BTreeSet<PublicKey>,
        deploy_hash: DeployHash,
    ) -> Self {
        DeployItem {
            public_key,
            session,
            payment,
            gas_price,
            authorization_keys,
            deploy_hash,
        }
    }
}
