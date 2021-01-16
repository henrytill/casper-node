#![no_std]
#![no_main]

use casper_contract::contract_api::{runtime, system};
use casper_types::{system_contract_errors::mint, ApiError, SecretKey, U512};

const ARG_AMOUNT: &str = "amount";

#[no_mangle]
pub extern "C" fn call() {
    let amount: U512 = runtime::get_named_arg(ARG_AMOUNT);
    let account = SecretKey::ed25519([42; SecretKey::ED25519_LENGTH]).into();
    let result = system::transfer_to_account(account, amount, None);
    let expected_error: ApiError = mint::Error::InsufficientFunds.into();
    assert_eq!(result, Err(expected_error))
}
