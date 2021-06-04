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
const CONTRACT_CALL_RECURSIVE_SUBCALL: &str = "ee_1217_call_recursive_subcall.wasm";

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

    assert_eq!(
        call_stack.len(),
        expected_call_stack_len,
        "call stack len was an unexpected size {}, should be {} {:#?}",
        call_stack.len(),
        expected_call_stack_len,
        call_stack,
    );

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

fn store_contract(builder: &mut WasmTestBuilder<InMemoryGlobalState>, session_filename: &str) {
    let store_contract_request =
        ExecuteRequestBuilder::standard(*DEFAULT_ACCOUNT_ADDR, session_filename, runtime_args! {})
            .build();
    builder
        .exec(store_contract_request)
        .commit()
        .expect_success();
}

fn run_forwarder_versioned_contract_by_name(depth_limit: usize) {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST);
    store_contract(&mut builder, CONTRACT_RECURSIVE_SUBCALL);

    let default_account = builder.get_account(*DEFAULT_ACCOUNT_ADDR).unwrap();

    let contract_package_hash: HashAddr = default_account
        .named_keys()
        .get(CONTRACT_PACKAGE_NAME)
        .cloned()
        .and_then(Key::into_hash)
        .unwrap();

    let call_forwarder_request = ExecuteRequestBuilder::versioned_contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_PACKAGE_NAME,
        None,
        CONTRACT_FORWARDER_ENTRYPOINT,
        runtime_args! {
            ARG_TARGET_CONTRACT_PACKAGE_HASH => contract_package_hash,
            ARG_TARGET_METHOD => CONTRACT_FORWARDER_ENTRYPOINT.to_string(),
            ARG_LIMIT => depth_limit as u8,
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
    for i in 0..depth_limit {
        assert_expected(
            &mut builder,
            &format!("forwarder-{}", i),
            *DEFAULT_ACCOUNT_ADDR,
            contract_package_hash.into(),
            i + 2,
            current_contract_hash,
        );
    }
}

fn run_forwarder_contract_by_name(depth_limit: usize) {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST);
    store_contract(&mut builder, CONTRACT_RECURSIVE_SUBCALL);

    let default_account = builder.get_account(*DEFAULT_ACCOUNT_ADDR).unwrap();

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
            ARG_LIMIT => depth_limit as u8,
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
    for i in 0..depth_limit {
        assert_expected(
            &mut builder,
            &format!("forwarder-{}", i),
            *DEFAULT_ACCOUNT_ADDR,
            contract_package_hash.into(),
            i + 2,
            current_contract_hash,
        );
    }
}

fn run_forwarder_versioned_contract_by_hash(depth_limit: usize) {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST);
    store_contract(&mut builder, CONTRACT_RECURSIVE_SUBCALL);

    let default_account = builder.get_account(*DEFAULT_ACCOUNT_ADDR).unwrap();

    let contract_package_hash: HashAddr = default_account
        .named_keys()
        .get(CONTRACT_PACKAGE_NAME)
        .cloned()
        .and_then(Key::into_hash)
        .unwrap();

    let call_forwarder_request = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        contract_package_hash.into(),
        None,
        CONTRACT_FORWARDER_ENTRYPOINT,
        runtime_args! {
            ARG_TARGET_CONTRACT_PACKAGE_HASH => contract_package_hash,
            ARG_TARGET_METHOD => CONTRACT_FORWARDER_ENTRYPOINT.to_string(),
            ARG_LIMIT => depth_limit as u8,
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
    for i in 0..depth_limit {
        assert_expected(
            &mut builder,
            &format!("forwarder-{}", i),
            *DEFAULT_ACCOUNT_ADDR,
            contract_package_hash.into(),
            i + 2,
            current_contract_hash,
        );
    }
}

fn run_forwarder_contract_by_hash(depth_limit: usize) {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST);
    store_contract(&mut builder, CONTRACT_RECURSIVE_SUBCALL);

    let default_account = builder.get_account(*DEFAULT_ACCOUNT_ADDR).unwrap();

    let contract_package_hash: HashAddr = default_account
        .named_keys()
        .get(CONTRACT_PACKAGE_NAME)
        .cloned()
        .and_then(Key::into_hash)
        .unwrap();

    // Pull out the contract hash from named keys manually rather than rely on the contract-by-name
    // feature.
    let stored_contract_hash: ContractHash = default_account
        .named_keys()
        .get(CONTRACT_NAME)
        .cloned()
        .and_then(Key::into_hash)
        .unwrap()
        .into();

    let call_forwarder_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        stored_contract_hash,
        CONTRACT_FORWARDER_ENTRYPOINT,
        runtime_args! {
            ARG_TARGET_CONTRACT_PACKAGE_HASH => contract_package_hash,
            ARG_TARGET_METHOD => CONTRACT_FORWARDER_ENTRYPOINT.to_string(),
            ARG_LIMIT => depth_limit as u8,
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
    for i in 0..depth_limit {
        assert_expected(
            &mut builder,
            &format!("forwarder-{}", i),
            *DEFAULT_ACCOUNT_ADDR,
            contract_package_hash.into(),
            i + 2,
            current_contract_hash,
        );
    }
}

fn run_forwarder_call_recursive_from_session_code(depth_limit: usize) {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST);
    store_contract(&mut builder, CONTRACT_RECURSIVE_SUBCALL);

    let default_account = builder.get_account(*DEFAULT_ACCOUNT_ADDR).unwrap();

    let contract_package_hash: HashAddr = default_account
        .named_keys()
        .get(CONTRACT_PACKAGE_NAME)
        .cloned()
        .and_then(Key::into_hash)
        .unwrap();

    let call_forwarder_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_CALL_RECURSIVE_SUBCALL,
        runtime_args! {
            ARG_TARGET_CONTRACT_PACKAGE_HASH => contract_package_hash,
            ARG_TARGET_METHOD => CONTRACT_FORWARDER_ENTRYPOINT.to_string(),
            ARG_LIMIT => depth_limit as u8,
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
    for i in 0..depth_limit {
        assert_expected(
            &mut builder,
            &format!("forwarder-{}", i),
            *DEFAULT_ACCOUNT_ADDR,
            contract_package_hash.into(),
            i + 2,
            current_contract_hash,
        );
    }
}

#[ignore]
#[test]
fn should_run_subcalls_with_callstack() {
    for depth_limit in &[1, 5, 10usize] {
        run_forwarder_contract_by_hash(*depth_limit);
        run_forwarder_contract_by_name(*depth_limit);
        run_forwarder_versioned_contract_by_hash(*depth_limit);
        run_forwarder_versioned_contract_by_name(*depth_limit);

        // TODO: fix this ->
        run_forwarder_call_recursive_from_session_code(*depth_limit);
    }
}
