#![no_std]
#![no_main]

extern crate alloc;

use casper_contract::contract_api::{account, runtime, system};
use casper_types::{
    auction::{self, DelegationRate},
    runtime_args, ContractHash, PublicKey, RuntimeArgs, SecretKey, URef, U512,
};

fn bond(contract_hash: ContractHash, bond_amount: U512, bonding_purse: URef) {
    let valid_public_key: PublicKey = SecretKey::ed25519([42; SecretKey::ED25519_LENGTH]).into();

    let runtime_args = runtime_args! {
        auction::ARG_PUBLIC_KEY => valid_public_key,
        auction::ARG_SOURCE_PURSE => bonding_purse,
        auction::ARG_DELEGATION_RATE => DelegationRate::from(42u8),
        auction::ARG_AMOUNT => bond_amount,
    };
    runtime::call_contract::<U512>(contract_hash, auction::METHOD_ADD_BID, runtime_args);
}

#[no_mangle]
pub extern "C" fn call() {
    // bond amount == 0 should fail
    bond(
        system::get_auction(),
        U512::from(0),
        account::get_main_purse(),
    );
}
