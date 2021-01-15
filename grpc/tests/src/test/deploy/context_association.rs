use casper_engine_test_support::{
    internal::{
        DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_PAYMENT,
        DEFAULT_RUN_GENESIS_REQUEST,
    },
    DEFAULT_ACCOUNT_PUBLIC_KEY,
};

use casper_types::{
    runtime_args,
    system_contract_type::{AUCTION, MINT, PROOF_OF_STAKE},
    RuntimeArgs,
};

const SYSTEM_CONTRACT_HASHES_WASM: &str = "system_contract_hashes.wasm";
const ARG_AMOUNT: &str = "amount";

#[ignore]
#[test]
fn should_put_system_contract_hashes_to_account_context() {
    let payment_purse_amount = *DEFAULT_PAYMENT;
    let mut builder = InMemoryWasmTestBuilder::default();

    let request = {
        let deploy = DeployItemBuilder::new()
            .with_address(*DEFAULT_ACCOUNT_PUBLIC_KEY)
            .with_session_code(SYSTEM_CONTRACT_HASHES_WASM, runtime_args! {})
            .with_empty_payment_bytes(runtime_args! { ARG_AMOUNT => payment_purse_amount})
            .with_authorization_keys(&[*DEFAULT_ACCOUNT_PUBLIC_KEY])
            .with_deploy_hash([1; 32])
            .build();

        ExecuteRequestBuilder::new().push_deploy(deploy).build()
    };

    builder
        .run_genesis(&DEFAULT_RUN_GENESIS_REQUEST)
        .exec(request)
        .expect_success()
        .commit();

    let account = builder
        .get_account(*DEFAULT_ACCOUNT_PUBLIC_KEY)
        .expect("account should exist");

    let named_keys = account.named_keys();

    assert!(named_keys.contains_key(MINT), "should contain mint");
    assert!(
        named_keys.contains_key(PROOF_OF_STAKE),
        "should contain proof of stake"
    );
    assert!(named_keys.contains_key(AUCTION), "should contain auction");

    assert_eq!(
        named_keys[MINT].into_hash().expect("should be a hash"),
        builder.get_mint_contract_hash(),
        "mint_contract_hash should match"
    );
    assert_eq!(
        named_keys[PROOF_OF_STAKE]
            .into_hash()
            .expect("should be a hash"),
        builder.get_pos_contract_hash(),
        "pos_contract_hash should match"
    );
    assert_eq!(
        named_keys[AUCTION].into_hash().expect("should be a hash"),
        builder.get_auction_contract_hash(),
        "auction_contract_hash should match"
    );
}
