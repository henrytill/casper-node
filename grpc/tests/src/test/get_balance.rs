use std::convert::TryFrom;

use once_cell::sync::Lazy;

use casper_engine_test_support::{
    internal::{ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_RUN_GENESIS_REQUEST},
    DEFAULT_ACCOUNT_PUBLIC_KEY,
};
use casper_execution_engine::{core, core::ValidationError, shared::newtypes::Blake2bHash};
use casper_types::{
    runtime_args, AccessRights, Key, PublicKey, RuntimeArgs, SecretKey, URef, U512,
};

const TRANSFER_ARG_TARGET: &str = "target";
const TRANSFER_ARG_AMOUNT: &str = "amount";
const TRANSFER_ARG_ID: &str = "id";

static ALICE_KEY: Lazy<PublicKey> =
    Lazy::new(|| SecretKey::ed25519([3; SecretKey::ED25519_LENGTH]).into());

static BOB_KEY: Lazy<PublicKey> =
    Lazy::new(|| SecretKey::ed25519([5; SecretKey::ED25519_LENGTH]).into());

static TRANSFER_AMOUNT_1: Lazy<U512> = Lazy::new(|| U512::from(100_000_000));

#[ignore]
#[test]
fn get_balance_should_work() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST);

    let transfer_request = ExecuteRequestBuilder::transfer(
        *DEFAULT_ACCOUNT_PUBLIC_KEY,
        runtime_args! {
            TRANSFER_ARG_TARGET => *ALICE_KEY,
            TRANSFER_ARG_AMOUNT => *TRANSFER_AMOUNT_1,
            TRANSFER_ARG_ID => <Option<u64>>::None,
        },
    )
    .build();

    builder.exec(transfer_request).commit().expect_success();

    let alice_account = builder
        .get_account(*ALICE_KEY)
        .expect("should have Alice's account");

    let alice_main_purse = alice_account.main_purse();

    let alice_balance_result = builder.get_purse_balance_result(alice_main_purse);

    let alice_balance = alice_balance_result
        .motes()
        .cloned()
        .expect("should have motes");

    assert_eq!(alice_balance, *TRANSFER_AMOUNT_1);

    let state_root_hash = {
        let post_state_hash = builder.get_post_state_hash();
        Blake2bHash::try_from(post_state_hash.as_slice()).expect("should convert")
    };

    let (purse_proof, balance_proof) = alice_balance_result.proofs().expect("should have proofs");

    assert!(core::validate_balance_proof(
        &state_root_hash,
        &purse_proof,
        &balance_proof,
        alice_main_purse.into(),
        &alice_balance,
    )
    .is_ok());

    let bogus_key = Key::Hash([1u8; 32]);
    assert_eq!(
        core::validate_balance_proof(
            &state_root_hash,
            &purse_proof,
            &balance_proof,
            bogus_key.to_owned(),
            &alice_balance,
        ),
        Err(ValidationError::KeyIsNotAURef(bogus_key))
    );

    let bogus_uref: Key = Key::URef(URef::new([3u8; 32], AccessRights::READ_ADD_WRITE));
    assert_eq!(
        core::validate_balance_proof(
            &state_root_hash,
            &purse_proof,
            &balance_proof,
            bogus_uref,
            &alice_balance,
        ),
        Err(ValidationError::UnexpectedKey)
    );

    let bogus_hash = Blake2bHash::new(&[5u8; 32]);
    assert_eq!(
        core::validate_balance_proof(
            &bogus_hash,
            &purse_proof,
            &balance_proof,
            alice_main_purse.into(),
            &alice_balance,
        ),
        Err(ValidationError::InvalidProofHash)
    );

    assert_eq!(
        core::validate_balance_proof(
            &state_root_hash,
            &purse_proof,
            &purse_proof,
            alice_main_purse.into(),
            &alice_balance,
        ),
        Err(ValidationError::UnexpectedKey)
    );

    let bogus_motes = U512::from(1337);
    assert_eq!(
        core::validate_balance_proof(
            &state_root_hash,
            &purse_proof,
            &balance_proof,
            alice_main_purse.into(),
            &bogus_motes,
        ),
        Err(ValidationError::UnexpectedValue)
    );

    ////////////////////////////////////////////

    let transfer_request = ExecuteRequestBuilder::transfer(
        *DEFAULT_ACCOUNT_PUBLIC_KEY,
        runtime_args! {
            TRANSFER_ARG_TARGET => *BOB_KEY,
            TRANSFER_ARG_AMOUNT => *TRANSFER_AMOUNT_1,
            TRANSFER_ARG_ID => <Option<u64>>::None
        },
    )
    .build();

    builder.exec(transfer_request).commit().expect_success();

    let alice_account = builder
        .get_account(*ALICE_KEY)
        .expect("should have Alice's account");

    let alice_main_purse = alice_account.main_purse();

    let alice_balance_result_new = builder.get_purse_balance_result(alice_main_purse);

    let state_root_hash = {
        let post_state_hash = builder.get_post_state_hash();
        Blake2bHash::try_from(post_state_hash.as_slice()).expect("should convert")
    };

    let (_purse_proof_new, balance_proof_new) = alice_balance_result_new
        .proofs()
        .expect("should have proofs");

    assert_eq!(
        core::validate_balance_proof(
            &state_root_hash,
            &purse_proof,
            &balance_proof_new,
            alice_main_purse.into(),
            &alice_balance,
        ),
        Err(ValidationError::InvalidProofHash)
    );
}
