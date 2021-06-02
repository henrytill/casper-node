#![no_std]
#![no_main]

extern crate alloc;

use alloc::{
    string::{String, ToString},
    vec,
};

use casper_contract::contract_api::{runtime, storage};
use casper_types::{
    runtime_args, ApiError, CLType, CLValue, ContractPackageHash, EntryPoint, EntryPointAccess,
    EntryPointType, EntryPoints, HashAddr, Key, Parameter, RuntimeArgs, KEY_HASH_LENGTH,
};

const PACKAGE_NAME: &str = "forwarder";
const PACKAGE_ACCESS_KEY_NAME: &str = "forwarder_access";

const METHOD_FORWARDER_NAME: &str = "forwarder";

const ARG_TARGET_CONTRACT_HASH: &str = "target_contract_hash";
const ARG_TARGET_METHOD: &str = "target_method";
const ARG_LIMIT: &str = "limit";
const ARG_CURRENT_DEPTH: &str = "current_depth";

#[no_mangle]
pub extern "C" fn forwarder() {
    let target_contract_package_hash: HashAddr = runtime::get_named_arg(ARG_TARGET_CONTRACT_HASH);
    let target_method: String = runtime::get_named_arg(ARG_TARGET_METHOD);
    let limit: u8 = runtime::get_named_arg(ARG_LIMIT);
    let current_depth: u8 = runtime::get_named_arg(ARG_CURRENT_DEPTH);

    let call_stack = runtime::get_call_stack();
    let name = alloc::format!("forwarder-{}", current_depth);
    let call_stack_at = storage::new_uref(call_stack);

    runtime::put_key(&name, Key::URef(call_stack_at));

    if current_depth == limit {
        runtime::ret(CLValue::unit())
    }

    let args = runtime_args! {
        ARG_TARGET_CONTRACT_HASH => target_contract_package_hash,
        ARG_TARGET_METHOD => target_method.clone(),
        ARG_LIMIT => limit,
        ARG_CURRENT_DEPTH => current_depth + 1u8,
    };

    runtime::call_versioned_contract::<()>(
        ContractPackageHash::new(target_contract_package_hash),
        None,
        &target_method,
        args,
    );
}

#[no_mangle]
pub extern "C" fn call() {
    let entry_points = {
        let mut entry_points = EntryPoints::new();
        let entry_point = EntryPoint::new(
            METHOD_FORWARDER_NAME.to_string(),
            vec![
                Parameter::new(
                    ARG_TARGET_CONTRACT_HASH,
                    CLType::ByteArray(KEY_HASH_LENGTH as u32),
                ),
                Parameter::new(ARG_TARGET_METHOD, CLType::String),
                Parameter::new(ARG_LIMIT, CLType::U8),
                Parameter::new(ARG_CURRENT_DEPTH, CLType::U8),
            ],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );
        entry_points.add_entry_point(entry_point);
        entry_points
    };

    let (_contract_hash, _contract_version) = storage::new_contract(
        entry_points,
        None,
        Some(PACKAGE_NAME.to_string()),
        Some(PACKAGE_ACCESS_KEY_NAME.to_string()),
    );
}
