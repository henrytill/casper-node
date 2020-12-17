#![no_std]
#![no_main]

use casper_contract::contract_api::runtime;
use casper_types::{ApiError, PublicKey};

const ARG_ACCOUNT: &str = "account";

#[no_mangle]
pub extern "C" fn call() {
    let expected_caller: PublicKey = runtime::get_named_arg(ARG_ACCOUNT);
    let caller: PublicKey = runtime::get_caller();
    if expected_caller != caller {
        runtime::revert(ApiError::User(0))
    }
}
