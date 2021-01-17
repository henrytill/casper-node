#![no_std]
#![no_main]

use casper_contract::{
    contract_api::{account, runtime},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    account::{ActionType, Weight},
    SecretKey,
};

const ARG_KEY_MANAGEMENT_THRESHOLD: &str = "key_management_threshold";
const ARG_DEPLOYMENT_THRESHOLD: &str = "deployment_threshold";

#[no_mangle]
pub extern "C" fn call() {
    let account = SecretKey::ed25519([123; SecretKey::ED25519_LENGTH]).into();
    account::add_associated_key(account, Weight::new(254)).unwrap_or_revert();
    let key_management_threshold: Weight = runtime::get_named_arg(ARG_KEY_MANAGEMENT_THRESHOLD);
    let deployment_threshold: Weight = runtime::get_named_arg(ARG_DEPLOYMENT_THRESHOLD);

    account::set_action_threshold(ActionType::KeyManagement, key_management_threshold)
        .unwrap_or_revert();
    account::set_action_threshold(ActionType::Deployment, deployment_threshold).unwrap_or_revert();
}
