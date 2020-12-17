use once_cell::sync::Lazy;

use casper_engine_test_support::{
    internal::{
        utils, ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_PAYMENT,
        DEFAULT_RUN_GENESIS_REQUEST,
    },
    DEFAULT_ACCOUNT_PUBLIC_KEY,
};
use casper_execution_engine::shared::stored_value::StoredValue;
use casper_types::{runtime_args, Key, PublicKey, RuntimeArgs, SecretKey, U512};

const CONTRACT_CREATE: &str = "ee_572_regression_create.wasm";
const CONTRACT_ESCALATE: &str = "ee_572_regression_escalate.wasm";
const CONTRACT_TRANSFER: &str = "transfer_purse_to_account.wasm";
const CREATE: &str = "create";

static ACCOUNT_1_PUBLIC_KEY: Lazy<PublicKey> =
    Lazy::new(|| SecretKey::ed25519([1u8; SecretKey::ED25519_LENGTH]).into());
static ACCOUNT_2_PUBLIC_KEY: Lazy<PublicKey> =
    Lazy::new(|| SecretKey::ed25519([2u8; SecretKey::ED25519_LENGTH]).into());

#[ignore]
#[test]
fn should_run_ee_572_regression() {
    let account_amount: U512 = *DEFAULT_PAYMENT + U512::from(100);
    let account_1_creation_args = runtime_args! {
        "target" => *ACCOUNT_1_PUBLIC_KEY,
        "amount" => account_amount
    };
    let account_2_creation_args = runtime_args! {
        "target" => *ACCOUNT_2_PUBLIC_KEY,
        "amount" => account_amount,
    };

    // This test runs a contract that's after every call extends the same key with
    // more data
    let mut builder = InMemoryWasmTestBuilder::default();

    let exec_request_1 = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_PUBLIC_KEY,
        CONTRACT_TRANSFER,
        account_1_creation_args,
    )
    .build();
    let exec_request_2 = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_PUBLIC_KEY,
        CONTRACT_TRANSFER,
        account_2_creation_args.clone(),
    )
    .build();

    let exec_request_3 = ExecuteRequestBuilder::standard(
        *ACCOUNT_1_PUBLIC_KEY,
        CONTRACT_CREATE,
        account_2_creation_args,
    )
    .build();

    // Create Accounts
    builder
        .run_genesis(&DEFAULT_RUN_GENESIS_REQUEST)
        .exec(exec_request_1)
        .expect_success()
        .commit();

    builder.exec(exec_request_2).expect_success().commit();

    // Store the creation contract
    builder.exec(exec_request_3).expect_success().commit();

    let contract: Key = {
        let account = match builder.query(None, Key::Account(*ACCOUNT_1_PUBLIC_KEY), &[]) {
            Ok(StoredValue::Account(account)) => account,
            _ => panic!("Could not find account at: {:?}", *ACCOUNT_1_PUBLIC_KEY),
        };
        *account
            .named_keys()
            .get(CREATE)
            .expect("Could not find contract pointer")
    };

    let exec_request_4 = ExecuteRequestBuilder::standard(
        *ACCOUNT_2_PUBLIC_KEY,
        CONTRACT_ESCALATE,
        runtime_args! {
            "contract_hash" => contract.into_hash().expect("should be hash"),
        },
    )
    .build();

    // Attempt to forge a new URef with escalated privileges
    let response = builder
        .exec(exec_request_4)
        .get_exec_response(3)
        .expect("should have a response")
        .to_owned();

    let error_message = utils::get_error_message(response);

    assert!(error_message.contains("ForgedReference"), error_message);
}
