use once_cell::sync::Lazy;

use casper_engine_test_support::{
    internal::{utils, ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNTS},
    MINIMUM_ACCOUNT_CREATION_BALANCE,
};
use casper_execution_engine::{core::engine_state::GenesisAccount, shared::motes::Motes};
use casper_types::{
    system_contract_errors::auction, ApiError, PublicKey, RuntimeArgs, SecretKey, U512,
};

const CONTRACT_EE_597_REGRESSION: &str = "ee_597_regression.wasm";

static VALID_PUBLIC_KEY: Lazy<PublicKey> =
    Lazy::new(|| SecretKey::ed25519([42; SecretKey::ED25519_LENGTH]).into());
const VALID_BALANCE: u64 = MINIMUM_ACCOUNT_CREATION_BALANCE;

#[ignore]
#[test]
fn should_fail_when_bonding_amount_is_zero_ee_597_regression() {
    let accounts = {
        let mut tmp: Vec<GenesisAccount> = DEFAULT_ACCOUNTS.clone();
        let account = GenesisAccount::new(
            *VALID_PUBLIC_KEY,
            Motes::new(VALID_BALANCE.into()),
            Motes::new(U512::zero()),
        );
        tmp.push(account);
        tmp
    };

    let run_genesis_request = utils::create_run_genesis_request(accounts);

    let exec_request = ExecuteRequestBuilder::standard(
        *VALID_PUBLIC_KEY,
        CONTRACT_EE_597_REGRESSION,
        RuntimeArgs::default(),
    )
    .build();

    let result = InMemoryWasmTestBuilder::default()
        .run_genesis(&run_genesis_request)
        .exec(exec_request)
        .commit()
        .finish();

    let response = result
        .builder()
        .get_exec_response(0)
        .expect("should have a response")
        .to_owned();

    let error_message = utils::get_error_message(response);

    // Error::BondTooSmall => 5,
    assert!(
        error_message.contains(&format!(
            "{:?}",
            ApiError::from(auction::Error::BondTooSmall)
        )),
        error_message
    );
}
