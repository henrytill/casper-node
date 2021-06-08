#![no_std]
#![no_main]

extern crate alloc;

use alloc::{boxed::Box, string::ToString, vec};

use casper_contract::contract_api::{runtime, storage};
use casper_types::{CLType, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Parameter};

const CONTRACT_PACKAGE_NAME: &str = "forwarder";
const PACKAGE_ACCESS_KEY_NAME: &str = "forwarder_access";

const CONTRACT_NAME: &str = "our_contract_name";

const METHOD_FORWARDER_CONTRACT_NAME: &str = "forwarder_contract";

const ARG_CALLS: &str = "calls";
const ARG_CURRENT_DEPTH: &str = "current_depth";

#[no_mangle]
pub extern "C" fn forwarder_contract() {
    ee_1217_recursive_subcall::stuff()
}

#[no_mangle]
pub extern "C" fn forwarder_session() {
    ee_1217_recursive_subcall::stuff()
}

#[no_mangle]
pub extern "C" fn call() {
    let entry_points = {
        let mut entry_points = EntryPoints::new();
        let entry_point = EntryPoint::new(
            METHOD_FORWARDER_CONTRACT_NAME.to_string(),
            vec![
                Parameter::new(ARG_CALLS, CLType::List(Box::new(CLType::Any))),
                Parameter::new(ARG_CURRENT_DEPTH, CLType::U8),
            ],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );
        entry_points.add_entry_point(entry_point);
        entry_points
    };

    let (contract_hash, _contract_version) = storage::new_contract(
        entry_points,
        None,
        Some(CONTRACT_PACKAGE_NAME.to_string()),
        Some(PACKAGE_ACCESS_KEY_NAME.to_string()),
    );

    runtime::put_key(CONTRACT_NAME, contract_hash.into());
}
