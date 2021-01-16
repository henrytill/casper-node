use std::convert::TryFrom;

use once_cell::sync::Lazy;

use casper_engine_test_support::{
    internal::{
        ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_PUBLIC_KEY,
        DEFAULT_PAYMENT, DEFAULT_RUN_GENESIS_REQUEST,
    },
    DEFAULT_ACCOUNT_INITIAL_BALANCE,
};
use casper_types::{
    auction::ARG_AMOUNT, mint::ARG_TARGET, runtime_args, system_contract_errors::mint, ApiError,
    CLValue, PublicKey, RuntimeArgs, SecretKey, U512,
};

const CONTRACT_TRANSFER_PURSE_TO_ACCOUNT: &str = "transfer_purse_to_account.wasm";

static ACCOUNT_1_PUBLIC_KEY: Lazy<PublicKey> =
    Lazy::new(|| SecretKey::ed25519([42u8; SecretKey::ED25519_LENGTH]).into());
static ACCOUNT_1_INITIAL_FUND: Lazy<U512> = Lazy::new(|| *DEFAULT_PAYMENT + 42);

#[ignore]
#[test]
fn should_run_purse_to_account_transfer() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST);

    assert!(
        builder.get_account(*ACCOUNT_1_PUBLIC_KEY).is_none(),
        "new account shouldn't exist yet"
    );

    let exec_request_1 = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_PUBLIC_KEY,
        CONTRACT_TRANSFER_PURSE_TO_ACCOUNT,
        runtime_args! { ARG_TARGET => *ACCOUNT_1_PUBLIC_KEY, ARG_AMOUNT => *ACCOUNT_1_INITIAL_FUND },
    )
    .build();

    builder.exec(exec_request_1).expect_success().commit();

    let new_account = builder
        .get_account(*ACCOUNT_1_PUBLIC_KEY)
        .expect("new account should exist now");

    let balance = builder.get_purse_balance(new_account.main_purse());

    assert_eq!(
        balance, *ACCOUNT_1_INITIAL_FUND,
        "balance should equal transferred amount"
    );
}

#[ignore]
#[test]
fn should_fail_when_sending_too_much_from_purse_to_account() {
    let exec_request_1 = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_PUBLIC_KEY,
        CONTRACT_TRANSFER_PURSE_TO_ACCOUNT,
        runtime_args! { "target" => *ACCOUNT_1_PUBLIC_KEY, "amount" => U512::max_value() },
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::default();

    builder
        .run_genesis(&DEFAULT_RUN_GENESIS_REQUEST)
        .exec(exec_request_1)
        .expect_success()
        .commit()
        .finish();

    // Get transforms output for genesis account
    let default_account = builder
        .get_account(*DEFAULT_ACCOUNT_PUBLIC_KEY)
        .expect("should get genesis account");

    // Obtain main purse's balance
    let final_balance_key = default_account.named_keys()["final_balance"].normalize();
    let final_balance = CLValue::try_from(
        builder
            .query(None, final_balance_key, &[])
            .expect("should have final balance"),
    )
    .expect("should be a CLValue")
    .into_t::<U512>()
    .expect("should be U512");
    // When trying to send too much coins the balance is left unchanged
    assert_eq!(
        final_balance,
        U512::from(DEFAULT_ACCOUNT_INITIAL_BALANCE) - *DEFAULT_PAYMENT,
        "final balance incorrect"
    );

    // Get the `transfer_result` for a given account
    let transfer_result_key = default_account.named_keys()["transfer_result"].normalize();
    let transfer_result = CLValue::try_from(
        builder
            .query(None, transfer_result_key, &[])
            .expect("should have transfer result"),
    )
    .expect("should be a CLValue")
    .into_t::<String>()
    .expect("should be String");

    // Main assertion for the result of `transfer_from_purse_to_purse`
    let expected_error: ApiError = mint::Error::InsufficientFunds.into();
    assert_eq!(
        transfer_result,
        format!("{:?}", Result::<(), _>::Err(expected_error)),
        "Transfer Error incorrect"
    );
}
