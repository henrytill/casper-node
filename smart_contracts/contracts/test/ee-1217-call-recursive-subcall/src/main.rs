#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;

use casper_contract::contract_api::runtime;
use casper_types::{runtime_args, HashAddr, RuntimeArgs};

// const ARG_TARGET_CALL_VERSIONED: &str = "target_call_versioned";

const ARG_TARGET_CONTRACT_PACKAGE_HASH: &str = "target_contract_package_hash";
const ARG_TARGET_METHOD: &str = "target_method";
const ARG_LIMIT: &str = "limit";
const ARG_CURRENT_DEPTH: &str = "current_depth";

#[no_mangle]
pub extern "C" fn call() {
    // todo take param for call_versioned_contract vs call_contract
    // ARG_TARGET_CONTRACT_OR_PACKAGE_HASH
    //   let target_call_versioned: bool = runtime::get_named_arg(ARG_TARGET_CALL_VERSIONED);

    let target_contract_package_hash: HashAddr =
        runtime::get_named_arg(ARG_TARGET_CONTRACT_PACKAGE_HASH);

    let target_method: String = runtime::get_named_arg(ARG_TARGET_METHOD);
    let limit: u8 = runtime::get_named_arg(ARG_LIMIT);
    let current_depth: u8 = runtime::get_named_arg(ARG_CURRENT_DEPTH);

    runtime::call_versioned_contract(
        target_contract_package_hash.into(),
        None,
        &target_method,
        runtime_args! {
            ARG_TARGET_CONTRACT_PACKAGE_HASH => target_contract_package_hash,
            ARG_TARGET_METHOD => target_method.clone(),
            ARG_LIMIT => limit,
            ARG_CURRENT_DEPTH => current_depth,
        },
    )
}
