#![no_std]
#![no_main]

extern crate alloc;

use alloc::{string::ToString, vec};

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    contracts::{EntryPoint, EntryPointAccess, EntryPointType, EntryPoints},
    AccessRights, ApiError, CLType, CLValue, Key, URef, U512,
};

const VALUE_KEY_NAME: &str = "value";

const METHOD_INIT_NAME: &str = "init";
const METHOD_RETURN_UREF_NAME: &str = "return_uref";

const PACKAGE_HASH_KEY_NAME: &str = "offer_value";
const PACKAGE_ACCESS_KEY_NAME: &str = "offer_value_access";
const PACKAGE_CONTRACT_HASH_KEY_NAME: &str = "offer_value_hash";
const PACKAGE_CONTRACT_VERSION_KEY_NAME: &str = "contract_version";

/// This is "contract code"

#[no_mangle]
pub extern "C" fn init() {
    let value = storage::new_uref(U512::zero());
    runtime::put_key(VALUE_KEY_NAME, value.into());
}

#[no_mangle]
pub extern "C" fn return_uref() {
    let value: Key = runtime::get_key(VALUE_KEY_NAME).unwrap_or_revert();
    let ret: URef = value.into_uref().unwrap_or_revert();
    runtime::ret(CLValue::from_t(ret).unwrap_or_revert());
}

/// This is "session code"

#[no_mangle]
pub extern "C" fn call() {
    let entry_points = {
        let mut entry_points = EntryPoints::new();
        let entry_point_init = EntryPoint::new(
            METHOD_INIT_NAME.to_string(),
            vec![],       // init takes no arguments
            CLType::Unit, // init is called for effect
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );
        let entry_point_offer_value = EntryPoint::new(
            METHOD_RETURN_UREF_NAME.to_string(),
            vec![],       // return_uref takes no arguments
            CLType::URef, // return_uref returns a URef
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );
        entry_points.add_entry_point(entry_point_init);
        entry_points.add_entry_point(entry_point_offer_value);
        entry_points
    };

    let (contract_hash, contract_version) = storage::new_contract(
        entry_points,
        None,
        Some(PACKAGE_HASH_KEY_NAME.to_string()),
        Some(PACKAGE_ACCESS_KEY_NAME.to_string()),
    );

    runtime::put_key(PACKAGE_CONTRACT_HASH_KEY_NAME, contract_hash.into());

    runtime::put_key(
        PACKAGE_CONTRACT_VERSION_KEY_NAME,
        storage::new_uref(contract_version).into(),
    );
}
