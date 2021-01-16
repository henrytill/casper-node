use once_cell::sync::Lazy;

use casper_engine_test_support::{
    internal::{
        utils, DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_PAYMENT,
        DEFAULT_RUN_GENESIS_REQUEST,
    },
    DEFAULT_ACCOUNT_PUBLIC_KEY, MINIMUM_ACCOUNT_CREATION_BALANCE,
};
use casper_execution_engine::{core::engine_state::CONV_RATE, shared::motes::Motes};
use casper_types::{runtime_args, PublicKey, RuntimeArgs, SecretKey, U512};

const DO_NOTHING_WASM: &str = "do_nothing.wasm";
const CONTRACT_TRANSFER_TO_ACCOUNT: &str = "transfer_to_account_u512.wasm";
const TRANSFER_MAIN_PURSE_TO_NEW_PURSE_WASM: &str = "transfer_main_purse_to_new_purse.wasm";
const NAMED_PURSE_PAYMENT_WASM: &str = "named_purse_payment.wasm";
const ARG_TARGET: &str = "target";
const ARG_AMOUNT: &str = "amount";
const ARG_PURSE_NAME: &str = "purse_name";
const ARG_DESTINATION: &str = "destination";

static ACCOUNT_1_PUBLIC_KEY: Lazy<PublicKey> =
    Lazy::new(|| SecretKey::ed25519([42u8; SecretKey::ED25519_LENGTH]).into());

#[ignore]
#[test]
fn should_charge_non_main_purse() {
    // as account_1, create & fund a new purse and use that to pay for something
    // instead of account_1 main purse
    const TEST_PURSE_NAME: &str = "test-purse";

    let account_1_funding_amount = U512::from(MINIMUM_ACCOUNT_CREATION_BALANCE);

    let mut builder = InMemoryWasmTestBuilder::default();

    let setup_exec_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_PUBLIC_KEY,
        CONTRACT_TRANSFER_TO_ACCOUNT,
        runtime_args! { ARG_TARGET => *ACCOUNT_1_PUBLIC_KEY, ARG_AMOUNT => account_1_funding_amount },
    )
    .build();

    let create_purse_exec_request = ExecuteRequestBuilder::standard(
        *ACCOUNT_1_PUBLIC_KEY,
        TRANSFER_MAIN_PURSE_TO_NEW_PURSE_WASM,
        runtime_args! { ARG_DESTINATION => TEST_PURSE_NAME, ARG_AMOUNT => *DEFAULT_PAYMENT },
    )
    .build();

    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST);

    builder.exec(setup_exec_request).expect_success().commit();
    builder
        .exec(create_purse_exec_request)
        .expect_success()
        .commit();
    let transfer_result = builder.finish();

    // get account_1
    let account_1 = transfer_result
        .builder()
        .get_account(*ACCOUNT_1_PUBLIC_KEY)
        .expect("should have account");
    // get purse
    let purse_key = account_1.named_keys()[TEST_PURSE_NAME];
    let purse = purse_key.into_uref().expect("should have uref");

    let purse_starting_balance = builder.get_purse_balance(purse);

    assert_eq!(
        purse_starting_balance, *DEFAULT_PAYMENT,
        "purse should be funded with expected amount"
    );

    // should be able to pay for exec using new purse
    let account_payment_exec_request = {
        let deploy = DeployItemBuilder::new()
            .with_public_key(*ACCOUNT_1_PUBLIC_KEY)
            .with_session_code(DO_NOTHING_WASM, RuntimeArgs::default())
            .with_payment_code(
                NAMED_PURSE_PAYMENT_WASM,
                runtime_args! {
                    ARG_PURSE_NAME => TEST_PURSE_NAME,
                    ARG_AMOUNT => *DEFAULT_PAYMENT
                },
            )
            .with_authorization_keys(&[*ACCOUNT_1_PUBLIC_KEY])
            .with_deploy_hash([3; 32])
            .build();

        ExecuteRequestBuilder::new().push_deploy(deploy).build()
    };

    let transfer_result = builder
        .exec(account_payment_exec_request)
        .expect_success()
        .commit()
        .finish();

    let response = transfer_result
        .builder()
        .get_exec_response(2)
        .expect("there should be a response")
        .clone();

    let result = utils::get_success_result(&response);
    let gas = result.cost();
    let motes = Motes::from_gas(gas, CONV_RATE).expect("should have motes");

    let expected_resting_balance = *DEFAULT_PAYMENT - motes.value();

    let purse_final_balance = builder.get_purse_balance(purse);

    assert_eq!(
        purse_final_balance, expected_resting_balance,
        "purse resting balance should equal funding amount minus exec costs"
    );
}
