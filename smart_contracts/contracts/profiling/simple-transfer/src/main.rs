#![no_std]
#![no_main]

use casper_contract::{
    contract_api::{runtime, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{ApiError, PublicKey, TransferredTo, U512};

const ARG_TARGET: &str = "target";
const ARG_AMOUNT: &str = "amount";

#[repr(u16)]
enum Error {
    NonExistentAccount = 0,
}

#[no_mangle]
pub extern "C" fn call() {
    let target: PublicKey = runtime::get_named_arg(ARG_TARGET);
    let amount: U512 = runtime::get_named_arg(ARG_AMOUNT);
    match system::transfer_to_account(target, amount, None).unwrap_or_revert() {
        TransferredTo::NewAccount => {
            runtime::revert(ApiError::User(Error::NonExistentAccount as u16))
        }
        TransferredTo::ExistingAccount => (),
    }
}
