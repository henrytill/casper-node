use casper_engine_test_support::internal::{
    DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, ARG_AMOUNT,
    DEFAULT_ACCOUNT_PUBLIC_KEY, DEFAULT_PAYMENT, DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_types::{runtime_args, RuntimeArgs, SecretKey};

const CONTRACT_KEY_MANAGEMENT_THRESHOLDS: &str = "key_management_thresholds.wasm";

const ARG_STAGE: &str = "stage";

#[ignore]
#[test]
fn should_verify_key_management_permission_with_low_weight() {
    let exec_request_1 = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_PUBLIC_KEY,
        CONTRACT_KEY_MANAGEMENT_THRESHOLDS,
        runtime_args! { ARG_STAGE => String::from("init") },
    )
    .build();
    let exec_request_2 = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_PUBLIC_KEY,
        CONTRACT_KEY_MANAGEMENT_THRESHOLDS,
        runtime_args! { ARG_STAGE => String::from("test-permission-denied") },
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

#[ignore]
#[test]
fn should_verify_key_management_permission_with_sufficient_weight() {
    let exec_request_1 = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_PUBLIC_KEY,
        CONTRACT_KEY_MANAGEMENT_THRESHOLDS,
        runtime_args! { ARG_STAGE => String::from("init") },
    )
    .build();
    let exec_request_2 = {
        let deploy = DeployItemBuilder::new()
            .with_public_key(*DEFAULT_ACCOUNT_PUBLIC_KEY)
            .with_empty_payment_bytes(runtime_args! { ARG_AMOUNT => *DEFAULT_PAYMENT, })
            // This test verifies that all key management operations succeed
            .with_session_code(
                "key_management_thresholds.wasm",
                runtime_args! { ARG_STAGE => String::from("test-key-mgmnt-succeed") },
            )
            .with_deploy_hash([2u8; 32])
            .with_authorization_keys(&[
                *DEFAULT_ACCOUNT_PUBLIC_KEY,
                // Key [42; 32] is created in init stage
                SecretKey::ed25519([42; 32]).into(),
            ])
            .build();
        ExecuteRequestBuilder::from_deploy_item(deploy).build()
    };
    InMemoryWasmTestBuilder::default()
        .run_genesis(&DEFAULT_RUN_GENESIS_REQUEST)
        .exec(exec_request_1)
        .expect_success()
        .commit()
        .exec(exec_request_2)
        .expect_success()
        .commit();
}
