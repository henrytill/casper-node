#![no_std]
#![no_main]

use casper_contract::contract_api::{runtime, storage};
use casper_types::U512;

const KEY_LENGTH: usize = 32;

const ARG_KEY: &str = "key";
const ARG_VALUE: &str = "value";

#[no_mangle]
pub extern "C" fn call() {
    let key_bytes: [u8; KEY_LENGTH] = runtime::get_named_arg(ARG_KEY);
    let value: U512 = runtime::get_named_arg(ARG_VALUE);
    storage::write_local(key_bytes, value);
}
