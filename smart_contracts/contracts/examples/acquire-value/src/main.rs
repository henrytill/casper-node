#![no_std]
#![no_main]

use casper_contract::contract_api::{runtime, storage};
use casper_types::{RuntimeArgs, URef, U512};

const ARG_PACKAGE_HASH: &str = "offer_value";

const METHOD_RETURN_UREF_NAME: &str = "return_uref";

#[no_mangle]
pub extern "C" fn call() {
    let contract_package_hash = runtime::get_named_arg(ARG_PACKAGE_HASH);

    let value: URef = runtime::call_contract(
        contract_package_hash,
        METHOD_RETURN_UREF_NAME,
        RuntimeArgs::new(),
    );

    // runtime::put_key("value_key", value.into());

    storage::add(value, U512::one());
}
