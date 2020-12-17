#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;

use casper_contract::{
    contract_api::{account, runtime},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    account::{
        ActionType, AddKeyFailure, RemoveKeyFailure, SetThresholdFailure, UpdateKeyFailure, Weight,
    },
    ApiError, SecretKey,
};

const ARG_STAGE: &str = "stage";

#[no_mangle]
pub extern "C" fn call() {
    let stage: String = runtime::get_named_arg(ARG_STAGE);

    let account_1 = SecretKey::ed25519([1; SecretKey::ED25519_LENGTH]).into();
    let account_42 = SecretKey::ed25519([42; SecretKey::ED25519_LENGTH]).into();
    let account_43 = SecretKey::ed25519([43; SecretKey::ED25519_LENGTH]).into();
    let account_44 = SecretKey::ed25519([44; SecretKey::ED25519_LENGTH]).into();

    if stage == "init" {
        // executed with weight >= 1
        account::add_associated_key(account_42, Weight::new(100)).unwrap_or_revert();
        // this key will be used to test permission denied when removing keys with low
        // total weight
        account::add_associated_key(account_42, Weight::new(1)).unwrap_or_revert();
        account::add_associated_key(account_1, Weight::new(1)).unwrap_or_revert();
        account::set_action_threshold(ActionType::KeyManagement, Weight::new(101))
            .unwrap_or_revert();
    } else if stage == "test-permission-denied" {
        // Has to be executed with keys of total weight < 255
        match account::add_associated_key(account_44, Weight::new(1)) {
            Ok(_) => runtime::revert(ApiError::User(200)),
            Err(AddKeyFailure::PermissionDenied) => {}
            Err(_) => runtime::revert(ApiError::User(201)),
        }

        match account::update_associated_key(account_43, Weight::new(2)) {
            Ok(_) => runtime::revert(ApiError::User(300)),
            Err(UpdateKeyFailure::PermissionDenied) => {}
            Err(_) => runtime::revert(ApiError::User(301)),
        }
        match account::remove_associated_key(account_43) {
            Ok(_) => runtime::revert(ApiError::User(400)),
            Err(RemoveKeyFailure::PermissionDenied) => {}
            Err(_) => runtime::revert(ApiError::User(401)),
        }

        match account::set_action_threshold(ActionType::KeyManagement, Weight::new(255)) {
            Ok(_) => runtime::revert(ApiError::User(500)),
            Err(SetThresholdFailure::PermissionDeniedError) => {}
            Err(_) => runtime::revert(ApiError::User(501)),
        }
    } else if stage == "test-key-mgmnt-succeed" {
        // Has to be executed with keys of total weight >= 254
        account::add_associated_key(account_44, Weight::new(1)).unwrap_or_revert();
        // Updates [43;32] key weight created in init stage
        account::update_associated_key(account_44, Weight::new(2)).unwrap_or_revert();
        // Removes [43;32] key created in init stage
        account::remove_associated_key(account_44).unwrap_or_revert();
        // Sets action threshodl
        account::set_action_threshold(ActionType::KeyManagement, Weight::new(100))
            .unwrap_or_revert();
    } else {
        runtime::revert(ApiError::User(1))
    }
}
