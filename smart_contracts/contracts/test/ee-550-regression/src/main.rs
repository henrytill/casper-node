#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;

use casper_contract::{
    contract_api::{account, runtime},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    account::{ActionType, Weight},
    ApiError, SecretKey,
};

#[repr(u16)]
enum Error {
    AddKey1 = 0,
    AddKey2 = 1,
    SetActionThreshold = 2,
    RemoveKey = 3,
    UpdateKey = 4,
    UnknownPass = 5,
}

impl Into<ApiError> for Error {
    fn into(self) -> ApiError {
        ApiError::User(self as u16)
    }
}

const KEY_1_ADDR: [u8; 32] = [100; 32];
const KEY_2_ADDR: [u8; 32] = [101; 32];

const ARG_PASS: &str = "pass";

#[no_mangle]
pub extern "C" fn call() {
    let pass: String = runtime::get_named_arg(ARG_PASS);

    let account_1 = SecretKey::ed25519(KEY_1_ADDR).into();
    let account_2 = SecretKey::ed25519(KEY_2_ADDR).into();

    match pass.as_str() {
        "init_remove" => {
            account::add_associated_key(account_1, Weight::new(2))
                .unwrap_or_revert_with(Error::AddKey1);
            account::add_associated_key(account_2, Weight::new(255))
                .unwrap_or_revert_with(Error::AddKey2);
            account::set_action_threshold(ActionType::KeyManagement, Weight::new(254))
                .unwrap_or_revert_with(Error::SetActionThreshold);
        }
        "test_remove" => {
            // Deployed with two keys of weights 2 and 255 (total saturates at 255) to satisfy new
            // threshold
            account::remove_associated_key(account_1).unwrap_or_revert_with(Error::RemoveKey);
        }

        "init_update" => {
            account::add_associated_key(account_1, Weight::new(3))
                .unwrap_or_revert_with(Error::AddKey1);
            account::add_associated_key(account_2, Weight::new(255))
                .unwrap_or_revert_with(Error::AddKey2);
            account::set_action_threshold(ActionType::KeyManagement, Weight::new(254))
                .unwrap_or_revert_with(Error::SetActionThreshold);
        }
        "test_update" => {
            // Deployed with two keys of weights 3 and 255 (total saturates at 255) to satisfy new
            // threshold
            account::update_associated_key(account_1, Weight::new(1))
                .unwrap_or_revert_with(Error::UpdateKey);
        }
        _ => {
            runtime::revert(Error::UnknownPass);
        }
    }
}
