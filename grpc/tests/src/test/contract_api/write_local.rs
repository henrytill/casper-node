use once_cell::sync::Lazy;

use casper_engine_test_support::{
    internal::{utils, ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNTS},
    MINIMUM_ACCOUNT_CREATION_BALANCE,
};
use casper_execution_engine::{
    core::engine_state::GenesisAccount,
    shared::{account::Account, motes::Motes},
};
use casper_types::{
    account::AccountHash, runtime_args, Key, PublicKey, RuntimeArgs, SecretKey, U512,
};

const CONTRACT_WRITE_LOCAL: &str = "write_local.wasm";
const ARG_KEY: &str = "key";
const ARG_VALUE: &str = "value";

const KEY: [u8; 32] = [0u8; 32];

static BID_ACCOUNT_1_PK: Lazy<PublicKey> =
    Lazy::new(|| SecretKey::ed25519([204; SecretKey::ED25519_LENGTH]).into());
static BID_ACCOUNT_1_ADDR: Lazy<AccountHash> = Lazy::new(|| AccountHash::from(&*BID_ACCOUNT_1_PK));
const BID_ACCOUNT_1_BALANCE: u64 = MINIMUM_ACCOUNT_CREATION_BALANCE;
const BID_ACCOUNT_1_BOND: u64 = 0;

static BID_ACCOUNT_2_PK: Lazy<PublicKey> =
    Lazy::new(|| SecretKey::ed25519([206; SecretKey::ED25519_LENGTH]).into());
static BID_ACCOUNT_2_ADDR: Lazy<AccountHash> = Lazy::new(|| AccountHash::from(&*BID_ACCOUNT_2_PK));
const BID_ACCOUNT_2_BALANCE: u64 = MINIMUM_ACCOUNT_CREATION_BALANCE;
const BID_ACCOUNT_2_BOND: u64 = 0;

#[ignore]
#[test]
fn values_written_by_write_local_should_not_be_overwritten_by_other_users() {
    let run_genesis_request = {
        let accounts = {
            let mut tmp: Vec<GenesisAccount> = DEFAULT_ACCOUNTS.clone();
            let account_1 = GenesisAccount::new(
                *BID_ACCOUNT_1_PK,
                *BID_ACCOUNT_1_ADDR,
                Motes::new(BID_ACCOUNT_1_BALANCE.into()),
                Motes::new(BID_ACCOUNT_1_BOND.into()),
            );
            let account_2 = GenesisAccount::new(
                *BID_ACCOUNT_2_PK,
                *BID_ACCOUNT_2_ADDR,
                Motes::new(BID_ACCOUNT_2_BALANCE.into()),
                Motes::new(BID_ACCOUNT_2_BOND.into()),
            );
            tmp.push(account_1);
            tmp.push(account_2);
            tmp
        };
        utils::create_run_genesis_request(accounts)
    };

    let account_1_value = U512::from(42);
    let account_1_write = ExecuteRequestBuilder::standard(
        *BID_ACCOUNT_1_ADDR,
        CONTRACT_WRITE_LOCAL,
        runtime_args! { ARG_KEY => KEY, ARG_VALUE => account_1_value },
    )
    .build();

    let account_2_value = U512::from(666);
    let account_2_write = ExecuteRequestBuilder::standard(
        *BID_ACCOUNT_2_ADDR,
        CONTRACT_WRITE_LOCAL,
        runtime_args! { ARG_KEY => KEY, ARG_VALUE => account_2_value },
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&run_genesis_request);
    builder.exec(account_1_write).commit();
    builder.exec(account_2_write).commit();

    let stored_value = builder.query(None, Key::Hash(KEY), &[]).unwrap();
    let value: U512 = stored_value
        .as_cl_value()
        .cloned()
        .unwrap()
        .into_t()
        .unwrap();

    assert_eq!(account_1_value, value)
}

#[ignore]
#[test]
fn users_should_not_be_able_to_corrupt_the_mint() {
    let run_genesis_request = {
        let accounts = {
            let mut tmp: Vec<GenesisAccount> = DEFAULT_ACCOUNTS.clone();
            let account_1 = GenesisAccount::new(
                *BID_ACCOUNT_1_PK,
                *BID_ACCOUNT_1_ADDR,
                Motes::new(BID_ACCOUNT_1_BALANCE.into()),
                Motes::new(BID_ACCOUNT_1_BOND.into()),
            );
            let account_2 = GenesisAccount::new(
                *BID_ACCOUNT_2_PK,
                *BID_ACCOUNT_2_ADDR,
                Motes::new(BID_ACCOUNT_2_BALANCE.into()),
                Motes::new(BID_ACCOUNT_2_BOND.into()),
            );
            tmp.push(account_1);
            tmp.push(account_2);
            tmp
        };
        utils::create_run_genesis_request(accounts)
    };

    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&run_genesis_request);

    let stored_value = builder
        .query(None, Key::Account(*BID_ACCOUNT_2_ADDR), &[])
        .unwrap();
    let account_2: Account = stored_value.as_account().cloned().unwrap();

    let account_2_balance_initial = builder.get_purse_balance(account_2.main_purse());

    let account_1_value = U512::from(42);
    let account_1_write = ExecuteRequestBuilder::standard(
        *BID_ACCOUNT_1_ADDR,
        CONTRACT_WRITE_LOCAL,
        runtime_args! { ARG_KEY => account_2.main_purse().addr(), ARG_VALUE => account_1_value },
    )
    .build();

    builder.exec(account_1_write).commit();

    let account_2_balance = builder.get_purse_balance(account_2.main_purse());
    assert_eq!(account_2_balance, account_2_balance_initial);
}
