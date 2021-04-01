use once_cell::sync::Lazy;

use casper_engine_test_support::{
    internal::{ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_RUN_GENESIS_REQUEST},
    DEFAULT_ACCOUNT_ADDR, MINIMUM_ACCOUNT_CREATION_BALANCE,
};
use casper_execution_engine::core::engine_state::SYSTEM_ACCOUNT_ADDR;
use casper_types::{
    account::AccountHash, runtime_args, ContractHash, Key, PublicKey, RuntimeArgs, SecretKey, U512,
};

static ALICE: Lazy<PublicKey> =
    Lazy::new(|| SecretKey::ed25519([3; SecretKey::ED25519_LENGTH]).into());
static BOB: Lazy<PublicKey> =
    Lazy::new(|| SecretKey::ed25519([5; SecretKey::ED25519_LENGTH]).into());

static ALICE_ADDR: Lazy<AccountHash> = Lazy::new(|| AccountHash::from(&*ALICE));
static BOB_ADDR: Lazy<AccountHash> = Lazy::new(|| AccountHash::from(&*BOB));

const CONTRACT_TRANSFER_TO_ACCOUNT: &str = "transfer_to_account_u512.wasm";
const ARG_TARGET: &str = "target";
const ARG_AMOUNT: &str = "amount";

const CONTRACT_OFFER_VALUE: &str = "offer_value.wasm";
const CONTRACT_INIT_VALUE: &str = "init_value.wasm";
const CONTRACT_ACQUIRE_VALUE: &str = "acquire_value.wasm";

const TRANSFER_AMOUNT: u64 = MINIMUM_ACCOUNT_CREATION_BALANCE;

#[test]
fn should_run_offer_value_example() {
    let alice_fund_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_TRANSFER_TO_ACCOUNT,
        runtime_args! {
            ARG_TARGET => *ALICE_ADDR,
            ARG_AMOUNT => U512::from(TRANSFER_AMOUNT)
        },
    )
    .build();

    let bob_fund_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_TRANSFER_TO_ACCOUNT,
        runtime_args! {
            ARG_TARGET => *BOB_ADDR,
            ARG_AMOUNT => U512::from(TRANSFER_AMOUNT)
        },
    )
    .build();

    let post_genesis_requests = vec![alice_fund_request, bob_fund_request];

    let mut builder = InMemoryWasmTestBuilder::default();

    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST);

    for request in post_genesis_requests {
        builder.exec(request).commit().expect_success();
    }

    let store_contract_request =
        ExecuteRequestBuilder::standard(*ALICE_ADDR, CONTRACT_OFFER_VALUE, RuntimeArgs::new())
            .build();

    builder.exec(store_contract_request).commit();

    let init_value_request =
        ExecuteRequestBuilder::standard(*ALICE_ADDR, CONTRACT_INIT_VALUE, RuntimeArgs::new())
            .build();

    builder.exec(init_value_request).commit().expect_success();

    const PACKAGE_CONTRACT_HASH_KEY_NAME: &str = "offer_value_hash";
    const VALUE_KEY_NAME: &str = "value";
    let alice_account = builder.get_account(*ALICE_ADDR).unwrap();
    let contract_hash = alice_account
        .named_keys()
        .get(PACKAGE_CONTRACT_HASH_KEY_NAME)
        .cloned()
        .and_then(Key::into_hash)
        .map(ContractHash::from)
        .unwrap();

    let value: U512 = builder.get_value(contract_hash, VALUE_KEY_NAME);

    assert_eq!(value, U512::zero());

    let acquire_value_request = ExecuteRequestBuilder::standard(
        *BOB_ADDR,
        CONTRACT_ACQUIRE_VALUE,
        runtime_args! {
            "offer_value" => contract_hash
        },
    )
    .build();

    builder
        .exec(acquire_value_request)
        .commit()
        .expect_success();

    let value: U512 = builder.get_value(contract_hash, VALUE_KEY_NAME);

    assert_eq!(value, U512::one());
}
