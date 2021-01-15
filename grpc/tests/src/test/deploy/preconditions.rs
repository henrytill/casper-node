use assert_matches::assert_matches;
use once_cell::sync::Lazy;

use casper_engine_test_support::{
    internal::{
        utils, DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder,
        DEFAULT_RUN_GENESIS_REQUEST,
    },
    DEFAULT_ACCOUNT_PUBLIC_KEY,
};
use casper_execution_engine::core::engine_state::Error;
use casper_types::{runtime_args, PublicKey, RuntimeArgs, SecretKey, U512};

const ARG_AMOUNT: &str = "amount";

static ACCOUNT_1_PUBLIC_KEY: Lazy<PublicKey> =
    Lazy::new(|| SecretKey::ed25519([1u8; SecretKey::ED25519_LENGTH]).into());

#[ignore]
#[test]
fn should_raise_precondition_authorization_failure_invalid_account() {
    let nonexistent_account = SecretKey::ed25519([99u8; SecretKey::ED25519_LENGTH]).into();
    let payment_purse_amount = 10_000_000;
    let transferred_amount = 1;

    let exec_request = {
        let deploy = DeployItemBuilder::new()
            .with_address(*DEFAULT_ACCOUNT_PUBLIC_KEY)
            .with_deploy_hash([1; 32])
            .with_session_code(
                "transfer_purse_to_account.wasm",
                runtime_args! { "target" => *ACCOUNT_1_PUBLIC_KEY, "amount" => U512::from(transferred_amount) },
            )
            .with_address(nonexistent_account)
            .with_empty_payment_bytes(runtime_args! { ARG_AMOUNT => U512::from(payment_purse_amount) })
            .with_authorization_keys(&[nonexistent_account])
            .build();

        ExecuteRequestBuilder::new().push_deploy(deploy).build()
    };

    let transfer_result = InMemoryWasmTestBuilder::default()
        .run_genesis(&DEFAULT_RUN_GENESIS_REQUEST)
        .exec(exec_request)
        .finish();

    let response = transfer_result
        .builder()
        .get_exec_response(0)
        .expect("there should be a response");

    let precondition_failure = utils::get_precondition_failure(response);
    assert_matches!(precondition_failure, Error::Authorization);
}

#[ignore]
#[test]
fn should_raise_precondition_authorization_failure_empty_authorized_keys() {
    let empty_keys: [PublicKey; 0] = [];
    let exec_request = {
        let deploy = DeployItemBuilder::new()
            .with_address(*DEFAULT_ACCOUNT_PUBLIC_KEY)
            .with_session_code("do_nothing.wasm", RuntimeArgs::default())
            .with_empty_payment_bytes(RuntimeArgs::default())
            .with_deploy_hash([1; 32])
            // empty authorization keys to force error
            .with_authorization_keys(&empty_keys)
            .build();

        ExecuteRequestBuilder::new().push_deploy(deploy).build()
    };

    let transfer_result = InMemoryWasmTestBuilder::default()
        .run_genesis(&DEFAULT_RUN_GENESIS_REQUEST)
        .exec(exec_request)
        .finish();

    let response = transfer_result
        .builder()
        .get_exec_response(0)
        .expect("there should be a response");

    let precondition_failure = utils::get_precondition_failure(response);
    assert_matches!(precondition_failure, Error::Authorization);
}

#[ignore]
#[test]
fn should_raise_precondition_authorization_failure_invalid_authorized_keys() {
    let account: PublicKey = SecretKey::ed25519([99u8; SecretKey::ED25519_LENGTH]).into();
    let payment_purse_amount = 10_000_000;
    let transferred_amount = 1;

    let exec_request = {
        let deploy = DeployItemBuilder::new()
            .with_address(*DEFAULT_ACCOUNT_PUBLIC_KEY)
            .with_deploy_hash([1; 32])
            .with_session_code(
                "transfer_purse_to_account.wasm",
                runtime_args! { "target" =>*ACCOUNT_1_PUBLIC_KEY, "amount" => U512::from(transferred_amount) },
            )
            .with_empty_payment_bytes(runtime_args! { ARG_AMOUNT => U512::from(payment_purse_amount) })
            // invalid authorization key to force error
            .with_authorization_keys(&[account])
            .build();

        ExecuteRequestBuilder::new().push_deploy(deploy).build()
    };

    let transfer_result = InMemoryWasmTestBuilder::default()
        .run_genesis(&DEFAULT_RUN_GENESIS_REQUEST)
        .exec(exec_request)
        .finish();

    let response = transfer_result
        .builder()
        .get_exec_response(0)
        .expect("there should be a response");

    let precondition_failure = utils::get_precondition_failure(response);
    assert_matches!(precondition_failure, Error::Authorization);
}
