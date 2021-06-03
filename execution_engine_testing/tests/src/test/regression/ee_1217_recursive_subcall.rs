use num_traits::One;

use casper_engine_test_support::{
    internal::{
        ExecuteRequestBuilder, InMemoryWasmTestBuilder, WasmTestBuilder,
        DEFAULT_RUN_GENESIS_REQUEST,
    },
    AccountHash, DEFAULT_ACCOUNT_ADDR,
};
use casper_execution_engine::{
    shared::stored_value::StoredValue, storage::global_state::in_memory::InMemoryGlobalState,
};
use casper_types::{
    runtime_args, system::CallStackElement, CLValue, ContractHash, ContractPackageHash, HashAddr,
    Key, RuntimeArgs,
};

const CONTRACT_RECURSIVE_SUBCALL: &str = "ee_1217_recursive_subcall.wasm";

const CONTRACT_PACKAGE_NAME: &str = "forwarder";
const CONTRACT_FORWARDER_ENTRYPOINT: &str = "forwarder";
const CONTRACT_NAME: &str = "our_contract_name";

const ARG_TARGET_CONTRACT_PACKAGE_HASH: &str = "target_contract_package_hash";
const ARG_TARGET_METHOD: &str = "target_method";
const ARG_LIMIT: &str = "limit";
const ARG_CURRENT_DEPTH: &str = "current_depth";

fn assert_expected(
    builder: &mut WasmTestBuilder<InMemoryGlobalState>,
    stored_call_stack_key: &str,
    expected_account_hash: AccountHash,
    expected_contract_package_hash: ContractPackageHash,
    expected_call_stack_len: usize,
    current_contract_hash: ContractHash,
) {
    let cl_value = builder
        .query(
            None,
            current_contract_hash.into(),
            &[stored_call_stack_key.to_string()],
        )
        .unwrap();

    let call_stack = cl_value
        .as_cl_value()
        .cloned()
        .map(CLValue::into_t::<Vec<CallStackElement>>)
        .unwrap()
        .unwrap();

    assert_eq!(call_stack.len(), expected_call_stack_len);

    let (head, rest) = call_stack.split_at(usize::one());

    assert_eq!(
        head,
        [CallStackElement::Session {
            account_hash: expected_account_hash
        }],
    );

    assert!(rest.windows(2).all(|w| match w {
        &[CallStackElement::Contract {
            contract_package_hash: left_package_hash,
            contract_hash: left_hash,
        }, CallStackElement::Contract {
            contract_package_hash: right_package_hash,
            contract_hash: right_hash,
        }] if left_package_hash == right_package_hash
            && left_package_hash == expected_contract_package_hash
            && left_hash == right_hash =>
            true,
        _ => false,
    }));
}

fn store_recursive_subcall_contract(builder: &mut WasmTestBuilder<InMemoryGlobalState>) {
    let store_contract_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_RECURSIVE_SUBCALL,
        runtime_args! {},
    )
    .build();
    builder
        .exec(store_contract_request)
        .commit()
        .expect_success();
}

#[ignore]
#[test]
fn should_call_forwarder_versioned_contract_by_name() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST);
    store_recursive_subcall_contract(&mut builder);

    let default_account = builder.get_account(*DEFAULT_ACCOUNT_ADDR).unwrap();

    let contract_package_hash: HashAddr = default_account
        .named_keys()
        .get(CONTRACT_PACKAGE_NAME)
        .cloned()
        .and_then(Key::into_hash)
        .unwrap();

    let call_forwarder_request = ExecuteRequestBuilder::versioned_contract_call_by_hash_key_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_PACKAGE_NAME,
        None,
        CONTRACT_FORWARDER_ENTRYPOINT,
        runtime_args! {
            ARG_TARGET_CONTRACT_PACKAGE_HASH => contract_package_hash,
            ARG_TARGET_METHOD => CONTRACT_FORWARDER_ENTRYPOINT.to_string(),
            ARG_LIMIT => 3u8,
            ARG_CURRENT_DEPTH => 0u8,
        },
    )
    .build();

    builder
        .exec(call_forwarder_request)
        .commit()
        .expect_success();

    let value = builder
        .query(None, Key::Hash(contract_package_hash), &[])
        .unwrap();

    let contract_package = match value {
        StoredValue::ContractPackage(package) => package,
        _ => panic!("unreachable"),
    };

    let current_contract_hash = contract_package.current_contract_hash().unwrap();
    assert_expected(
        &mut builder,
        "forwarder-0",
        *DEFAULT_ACCOUNT_ADDR,
        contract_package_hash.into(),
        2,
        current_contract_hash,
    );
}

#[ignore]
#[test]
fn should_call_forwarder_contract_by_name() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST);
    store_recursive_subcall_contract(&mut builder);

    let default_account = builder.get_account(*DEFAULT_ACCOUNT_ADDR).unwrap();

    println!("default_account: {:#?}", default_account);
    let contract_package_hash: HashAddr = default_account
        .named_keys()
        .get(CONTRACT_PACKAGE_NAME)
        .cloned()
        .and_then(Key::into_hash)
        .unwrap();

    let call_forwarder_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        CONTRACT_FORWARDER_ENTRYPOINT,
        runtime_args! {
            ARG_TARGET_CONTRACT_PACKAGE_HASH => contract_package_hash,
            ARG_TARGET_METHOD => CONTRACT_FORWARDER_ENTRYPOINT.to_string(),
            ARG_LIMIT => 3u8,
            ARG_CURRENT_DEPTH => 0u8,
        },
    )
    .build();

    builder
        .exec(call_forwarder_request)
        .commit()
        .expect_success();

    let value = builder
        .query(None, Key::Hash(contract_package_hash), &[])
        .unwrap();

    let contract_package = match value {
        StoredValue::ContractPackage(package) => package,
        _ => panic!("unreachable"),
    };

    let piqage = contract_package.current_contract_hash().unwrap();

    let cl_value = builder
        .query(None, piqage.into(), &["forwarder-0".to_string()])
        .unwrap();

    let call_stack = cl_value
        .as_cl_value()
        .cloned()
        .map(CLValue::into_t::<Vec<CallStackElement>>);

    println!("value {:?}", call_stack);
}
