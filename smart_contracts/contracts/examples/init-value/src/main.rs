#![no_std]
#![no_main]

use casper_contract::{contract_api::runtime, unwrap_or_revert::UnwrapOrRevert};
use casper_types::{ContractHash, Key, RuntimeArgs};

const PACKAGE_CONTRACT_HASH_KEY_NAME: &str = "offer_value_hash";

const METHOD_INIT_NAME: &str = "init";

#[no_mangle]
pub extern "C" fn call() {
    let contract_package_hash = runtime::get_key(PACKAGE_CONTRACT_HASH_KEY_NAME)
        .and_then(Key::into_hash)
        .map(ContractHash::from)
        .unwrap_or_revert();

    runtime::call_contract::<()>(contract_package_hash, METHOD_INIT_NAME, RuntimeArgs::new());
}
