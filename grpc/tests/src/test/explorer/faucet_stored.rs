use once_cell::sync::Lazy;

use casper_engine_test_support::internal::{
    utils, ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_PUBLIC_KEY,
    DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_types::{runtime_args, ApiError, PublicKey, RuntimeArgs, SecretKey, U512};

const FAUCET: &str = "faucet";
const CALL_FAUCET: &str = "call_faucet";
const ARG_TARGET: &str = "target";
const ARG_AMOUNT: &str = "amount";

static NEW_ACCOUNT_PUBLIC_KEY: Lazy<PublicKey> =
    Lazy::new(|| SecretKey::ed25519([99u8; SecretKey::ED25519_LENGTH]).into());

fn get_builder() -> InMemoryWasmTestBuilder {
    let mut builder = InMemoryWasmTestBuilder::default();
    {
        // first, store contract
        let store_request = ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_PUBLIC_KEY,
            &format!("{}_stored.wasm", FAUCET),
            runtime_args! {},
        )
        .build();

        builder.run_genesis(&*DEFAULT_RUN_GENESIS_REQUEST);
        builder.exec_commit_finish(store_request);
    }
    builder
}

#[ignore]
#[test]
fn should_get_funds_from_faucet_stored() {
    let mut builder = get_builder();

    let default_account = builder
        .get_account(*DEFAULT_ACCOUNT_PUBLIC_KEY)
        .expect("should have account");

    let contract_hash = default_account
        .named_keys()
        .get(FAUCET)
        .expect("contract_hash should exist")
        .into_hash()
        .expect("should be a hash");

    let amount = U512::from(1000);

    // call stored faucet
    let exec_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_PUBLIC_KEY,
        contract_hash,
        CALL_FAUCET,
        runtime_args! { ARG_TARGET => *NEW_ACCOUNT_PUBLIC_KEY, ARG_AMOUNT => amount },
    )
    .build();
    builder.exec(exec_request).expect_success().commit();

    let account = builder
        .get_account(*NEW_ACCOUNT_PUBLIC_KEY)
        .expect("should get account");

    let account_purse = account.main_purse();
    let account_balance = builder.get_purse_balance(account_purse);
    assert_eq!(
        account_balance, amount,
        "faucet should have created account with requested amount"
    );
}

#[ignore]
#[test]
fn should_fail_if_already_funded() {
    let mut builder = get_builder();

    let default_account = builder
        .get_account(*DEFAULT_ACCOUNT_PUBLIC_KEY)
        .expect("should have account");

    let contract_hash = default_account
        .named_keys()
        .get(FAUCET)
        .expect("contract_hash should exist")
        .into_hash()
        .expect("should be a hash");

    let amount = U512::from(1000);

    // call stored faucet
    let exec_request_1 = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_PUBLIC_KEY,
        contract_hash,
        CALL_FAUCET,
        runtime_args! { ARG_TARGET => *NEW_ACCOUNT_PUBLIC_KEY, ARG_AMOUNT => amount },
    )
    .build();

    builder.exec(exec_request_1).expect_success().commit();

    // call stored faucet again; should error
    let exec_request_2 = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_PUBLIC_KEY,
        contract_hash,
        CALL_FAUCET,
        runtime_args! { ARG_TARGET => *NEW_ACCOUNT_PUBLIC_KEY, ARG_AMOUNT => amount },
    )
    .build();

    builder.exec(exec_request_2);

    let exec_response = builder
        .get_exec_response(2)
        .expect("Expected to be called after run()");

    let error_message = utils::get_error_message(exec_response);
    assert!(
        error_message.contains(&format!("{:?}", ApiError::User(1))),
        "should have reverted with user error 1 (already funded) but received {}",
        error_message,
    );
}
