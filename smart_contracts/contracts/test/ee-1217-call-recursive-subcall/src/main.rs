#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;

use casper_contract::contract_api::runtime;
use casper_types::{runtime_args, ApiError, RuntimeArgs};

use ee_1217_recursive_subcall::{Call, ContractAddress};

const ARG_CALLS: &str = "calls";
const ARG_CURRENT_DEPTH: &str = "current_depth";

#[no_mangle]
pub extern "C" fn call() {
    let calls: Vec<Call> = runtime::get_named_arg(ARG_CALLS);
    let current_depth: u8 = runtime::get_named_arg(ARG_CURRENT_DEPTH);

    let args = runtime_args! {
        ARG_CALLS => calls.clone(),
        ARG_CURRENT_DEPTH => current_depth,
    };

    match calls.get(current_depth as usize) {
        Some(Call {
            contract_address: ContractAddress::ContractPackageHash(contract_package_hash),
            target_method,
        }) => {
            runtime::call_versioned_contract::<()>(
                *contract_package_hash,
                None,
                &target_method,
                args,
            );
        }
        Some(Call {
            contract_address: ContractAddress::ContractHash(contract_hash),
            target_method,
        }) => {
            runtime::call_contract::<()>(*contract_hash, &target_method, args);
        }
        _ => runtime::revert(ApiError::User(0)),
    }
}
