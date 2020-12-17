use std::{
    collections::BTreeSet,
    convert::{TryFrom, TryInto},
};

use casper_execution_engine::core::engine_state::deploy_item::DeployItem;
use casper_types::{bytesrepr, bytesrepr::ToBytes, DeployHash, PublicKey};

use crate::engine_server::{ipc, mappings::MappingError};

impl TryFrom<ipc::DeployItem> for DeployItem {
    type Error = MappingError;

    fn try_from(mut pb_deploy_item: ipc::DeployItem) -> Result<Self, Self::Error> {
        let address: PublicKey = bytesrepr::deserialize(pb_deploy_item.get_address().to_vec())
            .map_err(|_| MappingError::invalid_public_key())?;

        let session = pb_deploy_item
            .take_session()
            .payload
            .ok_or(MappingError::MissingPayload)?
            .try_into()?;

        let payment = pb_deploy_item
            .take_payment()
            .payload
            .ok_or(MappingError::MissingPayload)?
            .try_into()?;

        let gas_price = pb_deploy_item.get_gas_price();

        let authorization_keys = pb_deploy_item
            .get_authorization_keys()
            .iter()
            .map(|raw: &Vec<u8>| {
                bytesrepr::deserialize(raw.to_owned())
                    .map_err(|_| MappingError::invalid_public_key())
            })
            .collect::<Result<BTreeSet<PublicKey>, Self::Error>>()?;

        let deploy_hash =
            DeployHash::new(pb_deploy_item.get_deploy_hash().try_into().map_err(|_| {
                MappingError::invalid_deploy_hash_length(pb_deploy_item.deploy_hash.len())
            })?);

        Ok(DeployItem::new(
            address,
            session,
            payment,
            gas_price,
            authorization_keys,
            deploy_hash,
        ))
    }
}

impl From<DeployItem> for ipc::DeployItem {
    fn from(deploy_item: DeployItem) -> Self {
        let mut result = ipc::DeployItem::new();
        result.set_address(deploy_item.public_key.to_bytes().unwrap()); // TODO
        result.set_session(deploy_item.session.into());
        result.set_payment(deploy_item.payment.into());
        result.set_gas_price(deploy_item.gas_price);
        result.set_authorization_keys(
            deploy_item
                .authorization_keys
                .into_iter()
                .map(|key| key.to_bytes().unwrap()) // TODO
                .collect(),
        );
        result.set_deploy_hash(deploy_item.deploy_hash.value().to_vec());
        result
    }
}
