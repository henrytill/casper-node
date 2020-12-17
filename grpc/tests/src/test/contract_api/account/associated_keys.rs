use once_cell::sync::Lazy;

use casper_engine_test_support::internal::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_PUBLIC_KEY, DEFAULT_PAYMENT,
    DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_types::{account::Weight, runtime_args, PublicKey, RuntimeArgs, SecretKey, U512};

const CONTRACT_ADD_UPDATE_ASSOCIATED_KEY: &str = "add_update_associated_key.wasm";
const CONTRACT_REMOVE_ASSOCIATED_KEY: &str = "remove_associated_key.wasm";
const CONTRACT_TRANSFER_PURSE_TO_ACCOUNT: &str = "transfer_purse_to_account.wasm";
const ARG_ACCOUNT: &str = "account";

static ACCOUNT_1_PUBLIC_KEY: Lazy<PublicKey> =
    Lazy::new(|| SecretKey::ed25519([1u8; SecretKey::ED25519_LENGTH]).into());
static ACCOUNT_1_INITIAL_FUND: Lazy<U512> = Lazy::new(|| *DEFAULT_PAYMENT * 10);

#[ignore]
#[test]
fn should_manage_associated_key() {
    // for a given account, should be able to add a new associated key and update
    // that key
    let mut builder = InMemoryWasmTestBuilder::default();

    let exec_request_1 = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_PUBLIC_KEY,
        CONTRACT_TRANSFER_PURSE_TO_ACCOUNT,
        runtime_args! { "target" => *ACCOUNT_1_PUBLIC_KEY, "amount" => *ACCOUNT_1_INITIAL_FUND },
    )
    .build();
    let exec_request_2 = ExecuteRequestBuilder::standard(
        *ACCOUNT_1_PUBLIC_KEY,
        CONTRACT_ADD_UPDATE_ASSOCIATED_KEY,
        runtime_args! { "account" => *DEFAULT_ACCOUNT_PUBLIC_KEY, },
    )
    .build();

    builder
        .run_genesis(&DEFAULT_RUN_GENESIS_REQUEST)
        .exec(exec_request_1)
        .expect_success()
        .commit();

    builder.exec(exec_request_2).expect_success().commit();

    let genesis_key = *DEFAULT_ACCOUNT_PUBLIC_KEY;

    let account_1 = builder
        .get_account(*ACCOUNT_1_PUBLIC_KEY)
        .expect("should have account");

    let gen_weight = account_1
        .get_associated_key_weight(genesis_key)
        .expect("weight");

    let expected_weight = Weight::new(2);
    assert_eq!(*gen_weight, expected_weight, "unexpected weight");

    let exec_request_3 = ExecuteRequestBuilder::standard(
        *ACCOUNT_1_PUBLIC_KEY,
        CONTRACT_REMOVE_ASSOCIATED_KEY,
        runtime_args! { ARG_ACCOUNT => *DEFAULT_ACCOUNT_PUBLIC_KEY, },
    )
    .build();

    builder.exec(exec_request_3).expect_success().commit();

    let account_1 = builder
        .get_account(*ACCOUNT_1_PUBLIC_KEY)
        .expect("should have account");

    let new_weight = account_1.get_associated_key_weight(genesis_key);

    assert_eq!(new_weight, None, "key should be removed");

    let is_error = builder.is_error();
    assert!(!is_error);
}
