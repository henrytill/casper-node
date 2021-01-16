use once_cell::sync::Lazy;

use casper_engine_test_support::internal::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_PUBLIC_KEY, DEFAULT_PAYMENT,
    DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_types::{runtime_args, PublicKey, RuntimeArgs, SecretKey};

const CONTRACT_GET_CALLER: &str = "get_caller.wasm";
const CONTRACT_GET_CALLER_SUBCALL: &str = "get_caller_subcall.wasm";
const CONTRACT_TRANSFER_PURSE_TO_ACCOUNT: &str = "transfer_purse_to_account.wasm";

static ACCOUNT_1_PUBLIC_KEY: Lazy<PublicKey> =
    Lazy::new(|| SecretKey::ed25519([1u8; SecretKey::ED25519_LENGTH]).into());

#[ignore]
#[test]
fn should_run_get_caller_contract() {
    InMemoryWasmTestBuilder::default()
        .run_genesis(&DEFAULT_RUN_GENESIS_REQUEST)
        .exec(
            ExecuteRequestBuilder::standard(
                *DEFAULT_ACCOUNT_PUBLIC_KEY,
                CONTRACT_GET_CALLER,
                runtime_args! {"account" => *DEFAULT_ACCOUNT_PUBLIC_KEY},
            )
            .build(),
        )
        .expect_success()
        .commit();
}

#[ignore]
#[test]
fn should_run_get_caller_contract_other_account() {
    let mut builder = InMemoryWasmTestBuilder::default();

    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST);

    builder
        .exec(
            ExecuteRequestBuilder::standard(
                *DEFAULT_ACCOUNT_PUBLIC_KEY,
                CONTRACT_TRANSFER_PURSE_TO_ACCOUNT,
                runtime_args! {"target" => *ACCOUNT_1_PUBLIC_KEY, "amount"=> *DEFAULT_PAYMENT},
            )
            .build(),
        )
        .expect_success()
        .commit();

    builder
        .exec(
            ExecuteRequestBuilder::standard(
                *ACCOUNT_1_PUBLIC_KEY,
                CONTRACT_GET_CALLER,
                runtime_args! {"account" => *ACCOUNT_1_PUBLIC_KEY},
            )
            .build(),
        )
        .expect_success()
        .commit();
}

#[ignore]
#[test]
fn should_run_get_caller_subcall_contract() {
    {
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST);

        builder
            .exec(
                ExecuteRequestBuilder::standard(
                    *DEFAULT_ACCOUNT_PUBLIC_KEY,
                    CONTRACT_GET_CALLER_SUBCALL,
                    runtime_args! {"account" => *DEFAULT_ACCOUNT_PUBLIC_KEY},
                )
                .build(),
            )
            .expect_success()
            .commit();
    }

    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&DEFAULT_RUN_GENESIS_REQUEST)
        .exec(
            ExecuteRequestBuilder::standard(
                *DEFAULT_ACCOUNT_PUBLIC_KEY,
                CONTRACT_TRANSFER_PURSE_TO_ACCOUNT,
                runtime_args! {"target" => *ACCOUNT_1_PUBLIC_KEY, "amount"=> *DEFAULT_PAYMENT},
            )
            .build(),
        )
        .expect_success()
        .commit();
    builder
        .exec(
            ExecuteRequestBuilder::standard(
                *ACCOUNT_1_PUBLIC_KEY,
                CONTRACT_GET_CALLER_SUBCALL,
                runtime_args! {"account" => *ACCOUNT_1_PUBLIC_KEY},
            )
            .build(),
        )
        .expect_success()
        .commit();
}
