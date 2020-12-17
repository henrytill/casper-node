#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;

use auction::{DelegationRate, METHOD_ADD_BID};
use casper_contract::{
    contract_api::{account, runtime, system},
    unwrap_or_revert::UnwrapOrRevert,
};

use casper_types::{auction, runtime_args, ApiError, PublicKey, RuntimeArgs, U512};

const ARG_AMOUNT: &str = "amount";
const ARG_ENTRY_POINT: &str = "entry_point";
const ARG_PUBLIC_KEY: &str = "public_key";
const TEST_BOND_FROM_MAIN_PURSE: &str = "bond-from-main-purse";
const TEST_SEED_NEW_ACCOUNT: &str = "seed_new_account";

#[repr(u16)]
enum Error {
    UnableToSeedAccount,
    UnknownCommand,
}

#[no_mangle]
pub extern "C" fn call() {
    let command: String = runtime::get_named_arg(ARG_ENTRY_POINT);

    match command.as_str() {
        TEST_BOND_FROM_MAIN_PURSE => bond_from_main_purse(),
        TEST_SEED_NEW_ACCOUNT => seed_new_account(),
        _ => runtime::revert(ApiError::User(Error::UnknownCommand as u16)),
    }
}

fn bond_from_main_purse() {
    let auction = system::get_auction();
    let bond_amount: U512 = runtime::get_named_arg(ARG_AMOUNT);
    let public_key: PublicKey = runtime::get_named_arg(ARG_PUBLIC_KEY);
    let args = runtime_args! {
        auction::ARG_PUBLIC_KEY => public_key,
        auction::ARG_SOURCE_PURSE => account::get_main_purse(),
        auction::ARG_DELEGATION_RATE => DelegationRate::from(42u8),
        auction::ARG_AMOUNT => bond_amount,
    };
    let _amount: U512 = runtime::call_contract(auction, METHOD_ADD_BID, args);
}

fn seed_new_account() {
    let source = account::get_main_purse();
    let target: PublicKey = runtime::get_named_arg(ARG_PUBLIC_KEY);
    let amount: U512 = runtime::get_named_arg(ARG_AMOUNT);
    system::transfer_from_purse_to_account(source, target, amount, None)
        .unwrap_or_revert_with(ApiError::User(Error::UnableToSeedAccount as u16));
}
