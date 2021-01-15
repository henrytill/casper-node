use once_cell::sync::Lazy;

use casper_engine_test_support::{
    internal::{
        DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, ARG_AMOUNT,
        DEFAULT_PAYMENT, DEFAULT_RUN_GENESIS_REQUEST,
    },
    DEFAULT_ACCOUNT_PUBLIC_KEY,
};
use casper_types::{runtime_args, PublicKey, RuntimeArgs, SecretKey};

const PASS_INIT_REMOVE: &str = "init_remove";
const PASS_TEST_REMOVE: &str = "test_remove";
const PASS_INIT_UPDATE: &str = "init_update";
const PASS_TEST_UPDATE: &str = "test_update";

const CONTRACT_EE_550_REGRESSION: &str = "ee_550_regression.wasm";
const DEPLOY_HASH: [u8; 32] = [42; 32];
const ARG_PASS: &str = "pass";

static KEY_2: Lazy<PublicKey> =
    Lazy::new(|| SecretKey::ed25519([101; SecretKey::ED25519_LENGTH]).into());

#[ignore]
#[test]
fn should_run_ee_550_remove_with_saturated_threshold_regression() {
    let exec_request_1 = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_PUBLIC_KEY,
        CONTRACT_EE_550_REGRESSION,
        runtime_args! { ARG_PASS => String::from(PASS_INIT_REMOVE) },
    )
    .build();

    let exec_request_2 = {
        let deploy_item = DeployItemBuilder::new()
            .with_address(*DEFAULT_ACCOUNT_PUBLIC_KEY)
            .with_session_code(
                CONTRACT_EE_550_REGRESSION,
                runtime_args! { ARG_PASS => String::from(PASS_TEST_REMOVE) },
            )
            .with_empty_payment_bytes(runtime_args! { ARG_AMOUNT => *DEFAULT_PAYMENT, })
            .with_authorization_keys(&[*DEFAULT_ACCOUNT_PUBLIC_KEY, *KEY_2])
            .with_deploy_hash(DEPLOY_HASH)
            .build();

        ExecuteRequestBuilder::from_deploy_item(deploy_item).build()
    };

    let mut builder = InMemoryWasmTestBuilder::default();

    builder
        .run_genesis(&DEFAULT_RUN_GENESIS_REQUEST)
        .exec(exec_request_1)
        .expect_success()
        .commit()
        .exec(exec_request_2)
        .expect_success()
        .commit();
}

#[ignore]
#[test]
fn should_run_ee_550_update_with_saturated_threshold_regression() {
    let exec_request_1 = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_PUBLIC_KEY,
        CONTRACT_EE_550_REGRESSION,
        runtime_args! { ARG_PASS => String::from(PASS_INIT_UPDATE) },
    )
    .build();

    let exec_request_2 = {
        let deploy_item = DeployItemBuilder::new()
            .with_address(*DEFAULT_ACCOUNT_PUBLIC_KEY)
            .with_session_code(
                CONTRACT_EE_550_REGRESSION,
                runtime_args! { ARG_PASS => String::from(PASS_TEST_UPDATE) },
            )
            .with_empty_payment_bytes(runtime_args! { ARG_AMOUNT => *DEFAULT_PAYMENT, })
            .with_authorization_keys(&[*DEFAULT_ACCOUNT_PUBLIC_KEY, *KEY_2])
            .with_deploy_hash(DEPLOY_HASH)
            .build();

        ExecuteRequestBuilder::from_deploy_item(deploy_item).build()
    };

    let mut builder = InMemoryWasmTestBuilder::default();

    builder
        .run_genesis(&DEFAULT_RUN_GENESIS_REQUEST)
        .exec(exec_request_1)
        .expect_success()
        .commit()
        .exec(exec_request_2)
        .expect_success()
        .commit();
}
