use once_cell::sync::Lazy;

use casper_engine_test_support::{
    internal::{
        ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_PAYMENT,
        DEFAULT_RUN_GENESIS_REQUEST,
    },
    DEFAULT_ACCOUNT_PUBLIC_KEY,
};
use casper_execution_engine::shared::stored_value::StoredValue;
use casper_types::{runtime_args, Key, PublicKey, RuntimeArgs, SecretKey};

const CONTRACT_MAIN_PURSE: &str = "main_purse.wasm";
const CONTRACT_TRANSFER_PURSE_TO_ACCOUNT: &str = "transfer_purse_to_account.wasm";
const ARG_TARGET: &str = "target";
const ARG_AMOUNT: &str = "amount";

static ACCOUNT_1_PUBLIC_KEY: Lazy<PublicKey> =
    Lazy::new(|| SecretKey::ed25519([1u8; SecretKey::ED25519_LENGTH]).into());

#[ignore]
#[test]
fn should_run_main_purse_contract_default_account() {
    let mut builder = InMemoryWasmTestBuilder::default();

    let builder = builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST);

    let default_account = if let Ok(StoredValue::Account(account)) =
        builder.query(None, Key::Account(*DEFAULT_ACCOUNT_PUBLIC_KEY), &[])
    {
        account
    } else {
        panic!("could not get account")
    };

    let exec_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_PUBLIC_KEY,
        CONTRACT_MAIN_PURSE,
        runtime_args! { "purse" => default_account.main_purse() },
    )
    .build();

    builder.exec(exec_request).expect_success().commit();
}

#[ignore]
#[test]
fn should_run_main_purse_contract_account_1() {
    let mut builder = InMemoryWasmTestBuilder::default();

    let exec_request_1 = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_PUBLIC_KEY,
        CONTRACT_TRANSFER_PURSE_TO_ACCOUNT,
        runtime_args! { ARG_TARGET => *ACCOUNT_1_PUBLIC_KEY, ARG_AMOUNT => *DEFAULT_PAYMENT },
    )
    .build();

    let builder = builder
        .run_genesis(&DEFAULT_RUN_GENESIS_REQUEST)
        .exec(exec_request_1)
        .expect_success()
        .commit();

    let account_1 = builder
        .get_account(*ACCOUNT_1_PUBLIC_KEY)
        .expect("should get account");

    let exec_request_2 = ExecuteRequestBuilder::standard(
        *ACCOUNT_1_PUBLIC_KEY,
        CONTRACT_MAIN_PURSE,
        runtime_args! { "purse" => account_1.main_purse() },
    )
    .build();

    builder.exec(exec_request_2).expect_success().commit();
}
