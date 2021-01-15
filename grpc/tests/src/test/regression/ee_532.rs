use once_cell::sync::Lazy;

use casper_engine_test_support::internal::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_execution_engine::core::engine_state::Error;
use casper_types::{PublicKey, RuntimeArgs, SecretKey};

const CONTRACT_EE_532_REGRESSION: &str = "ee_532_regression.wasm";

static UNKNOWN_PUBLIC_KEY: Lazy<PublicKey> =
    Lazy::new(|| SecretKey::ed25519([42u8; SecretKey::ED25519_LENGTH]).into());

#[ignore]
#[test]
fn should_run_ee_532_get_uref_regression_test() {
    // This test runs a contract that's after every call extends the same key with
    // more data

    let exec_request = ExecuteRequestBuilder::standard(
        *UNKNOWN_PUBLIC_KEY,
        CONTRACT_EE_532_REGRESSION,
        RuntimeArgs::default(),
    )
    .build();

    let result = InMemoryWasmTestBuilder::default()
        .run_genesis(&DEFAULT_RUN_GENESIS_REQUEST)
        .exec(exec_request)
        .commit()
        .finish();

    let deploy_result = result
        .builder()
        .get_exec_response(0)
        .expect("should have exec response")
        .get(0)
        .expect("should have at least one deploy result");

    assert!(
        deploy_result.has_precondition_failure(),
        "expected precondition failure"
    );

    let message = deploy_result.as_error().map(|err| format!("{}", err));
    assert_eq!(
        message,
        Some(format!("{}", Error::Authorization)),
        "expected Error::Authorization"
    )
}
