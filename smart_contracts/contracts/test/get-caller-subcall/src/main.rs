#![no_std]
#![no_main]

extern crate alloc;

use alloc::{string::ToString, vec::Vec};

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    ApiError, CLType, CLValue, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints,
    PublicKey, RuntimeArgs,
};

const ENTRY_POINT_NAME: &str = "get_caller_ext";
const HASH_KEY_NAME: &str = "caller_subcall";
const ACCESS_KEY_NAME: &str = "caller_subcall_access";
const ARG_ACCOUNT: &str = "account";

#[no_mangle]
pub extern "C" fn get_caller_ext() {
    let caller: PublicKey = runtime::get_caller();
    runtime::ret(CLValue::from_t(caller).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn call() {
    let expected_caller: PublicKey = runtime::get_named_arg(ARG_ACCOUNT);
    let caller: PublicKey = runtime::get_caller();
    if expected_caller != caller {
        runtime::revert(ApiError::User(0))
    }

    let entry_points = {
        let mut entry_points = EntryPoints::new();
        // takes no args, ret's PublicKey
        let entry_point = EntryPoint::new(
            ENTRY_POINT_NAME.to_string(),
            Vec::new(),
            CLType::ByteArray(32),
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );
        entry_points.add_entry_point(entry_point);
        entry_points
    };

    let (contract_hash, _contract_version) = storage::new_contract(
        entry_points,
        None,
        Some(HASH_KEY_NAME.to_string()),
        Some(ACCESS_KEY_NAME.to_string()),
    );

    let subcall_caller: PublicKey =
        runtime::call_contract(contract_hash, ENTRY_POINT_NAME, RuntimeArgs::default());
    if expected_caller != subcall_caller {
        runtime::revert(ApiError::User(1))
    }
}
