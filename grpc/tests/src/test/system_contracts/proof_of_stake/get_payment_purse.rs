use once_cell::sync::Lazy;

use casper_engine_test_support::{
    internal::{
        ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_PAYMENT,
        DEFAULT_RUN_GENESIS_REQUEST,
    },
    DEFAULT_ACCOUNT_PUBLIC_KEY, MINIMUM_ACCOUNT_CREATION_BALANCE,
};
use casper_types::{runtime_args, PublicKey, RuntimeArgs, SecretKey, U512};

const CONTRACT_POS_GET_PAYMENT_PURSE: &str = "pos_get_payment_purse.wasm";
const CONTRACT_TRANSFER_PURSE_TO_ACCOUNT: &str = "transfer_purse_to_account.wasm";
const ACCOUNT_1_INITIAL_BALANCE: u64 = MINIMUM_ACCOUNT_CREATION_BALANCE;
const ARG_AMOUNT: &str = "amount";
const ARG_TARGET: &str = "target";

static ACCOUNT_1_PUBLIC_KEY: Lazy<PublicKey> =
    Lazy::new(|| SecretKey::ed25519([1u8; SecretKey::ED25519_LENGTH]).into());

#[ignore]
#[test]
fn should_run_get_payment_purse_contract_default_account() {
    let exec_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_PUBLIC_KEY,
        CONTRACT_POS_GET_PAYMENT_PURSE,
        runtime_args! {
            ARG_AMOUNT => *DEFAULT_PAYMENT,
        },
    )
    .build();
    InMemoryWasmTestBuilder::default()
        .run_genesis(&DEFAULT_RUN_GENESIS_REQUEST)
        .exec(exec_request)
        .expect_success()
        .commit();
}

#[ignore]
#[test]
fn should_run_get_payment_purse_contract_account_1() {
    let exec_request_1 = ExecuteRequestBuilder::standard(
       *DEFAULT_ACCOUNT_PUBLIC_KEY,
        CONTRACT_TRANSFER_PURSE_TO_ACCOUNT,
        runtime_args! { ARG_TARGET => *ACCOUNT_1_PUBLIC_KEY, ARG_AMOUNT => U512::from(ACCOUNT_1_INITIAL_BALANCE) },
    )
        .build();
    let exec_request_2 = ExecuteRequestBuilder::standard(
        *ACCOUNT_1_PUBLIC_KEY,
        CONTRACT_POS_GET_PAYMENT_PURSE,
        runtime_args! {
            ARG_AMOUNT => *DEFAULT_PAYMENT,
        },
    )
    .build();
    InMemoryWasmTestBuilder::default()
        .run_genesis(&DEFAULT_RUN_GENESIS_REQUEST)
        .exec(exec_request_1)
        .expect_success()
        .commit()
        .exec(exec_request_2)
        .expect_success()
        .commit();
}
