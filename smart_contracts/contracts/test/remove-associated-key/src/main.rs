#![no_std]
#![no_main]

use casper_contract::{
    contract_api::{account, runtime},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{ApiError, PublicKey};

const ARG_ACCOUNT: &str = "account";

#[no_mangle]
pub extern "C" fn call() {
    let account: PublicKey = runtime::get_named_arg(ARG_ACCOUNT);
    account::remove_associated_key(account).unwrap_or_revert_with(ApiError::User(0))
}
