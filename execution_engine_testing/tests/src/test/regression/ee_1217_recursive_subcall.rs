use num_traits::One;
use rand::Rng;

use ee_1217_recursive_subcall::{Call, ContractAddress};

use casper_engine_test_support::{
    internal::{
        DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, WasmTestBuilder,
        DEFAULT_RUN_GENESIS_REQUEST,
    },
    AccountHash, DEFAULT_ACCOUNT_ADDR,
};
use casper_execution_engine::{
    shared::{stored_value::StoredValue, wasm},
    storage::global_state::in_memory::InMemoryGlobalState,
};
use casper_types::{
    runtime_args, system::CallStackElement, CLValue, ContractHash, ContractPackageHash,
    EntryPointType, HashAddr, Key, RuntimeArgs,
};

const CONTRACT_RECURSIVE_SUBCALL: &str = "ee_1217_recursive_subcall.wasm";
const CONTRACT_CALL_RECURSIVE_SUBCALL: &str = "ee_1217_call_recursive_subcall.wasm";

const CONTRACT_PACKAGE_NAME: &str = "forwarder";
const CONTRACT_FORWARDER_ENTRYPOINT_CONTRACT: &str = "forwarder_contract";

#[allow(dead_code)]
const CONTRACT_FORWARDER_ENTRYPOINT_SESSION: &str = "forwarder_session";

const CONTRACT_NAME: &str = "our_contract_name";

const ARG_CALLS: &str = "calls";
const ARG_CURRENT_DEPTH: &str = "current_depth";

fn assert_expected(
    builder: &mut WasmTestBuilder<InMemoryGlobalState>,
    stored_call_stack_key: &str,
    expected_account_hash: AccountHash,
    expected_call_stack_len: usize,
    current_key: Key,
    assertion: impl FnOnce(&[CallStackElement]) -> bool,
) {
    let cl_value = builder
        .query(None, current_key, &[stored_call_stack_key.to_string()])
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

    assert!(assertion(rest), "{:#?}", rest);
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

fn assert_all_elements_are_the_same_stored_contract(
    expected_contract_package_hash: ContractPackageHash,
) -> impl FnOnce(&[CallStackElement]) -> bool {
    move |rest| {
        rest.windows(2).all(|w| match w {
            &[CallStackElement::StoredContract {
                contract_package_hash: left_package_hash,
                contract_hash: left_hash,
            }, CallStackElement::StoredContract {
                contract_package_hash: right_package_hash,
                contract_hash: right_hash,
            }] if left_package_hash == right_package_hash
                && left_package_hash == expected_contract_package_hash
                && left_hash == right_hash =>
            {
                true
            }
            _ => false,
        })
    }
}

fn assert_all_elements_are_the_same_stored_session(
    expected_account_hash: AccountHash,
    expected_contract_package_hash: ContractPackageHash,
) -> impl FnOnce(&[CallStackElement]) -> bool {
    move |rest| {
        rest.windows(2).all(|w| match w {
            &[CallStackElement::StoredSession {
                account_hash: left_account_hash,
                contract_package_hash: left_package_hash,
                contract_hash: left_hash,
            }, CallStackElement::StoredSession {
                account_hash: right_account_hash,
                contract_package_hash: right_package_hash,
                contract_hash: right_hash,
            }] if left_account_hash == right_account_hash
                && left_account_hash == expected_account_hash
                && left_package_hash == right_package_hash
                && left_package_hash == expected_contract_package_hash
                && left_hash == right_hash =>
            {
                true
            }
            _ => false,
        })
    }
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

    let calls = vec![
        Call {
            contract_address: ContractAddress::ContractPackageHash(contract_package_hash.into()),
            target_method: "forwarder_contract".to_string(),
            entry_point_type: EntryPointType::Contract
        };
        depth_limit
    ];

    let call_forwarder_request = ExecuteRequestBuilder::versioned_contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_PACKAGE_NAME,
        None,
        CONTRACT_FORWARDER_ENTRYPOINT_CONTRACT,
        runtime_args! {
            ARG_CALLS => calls,
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
            i + 2,
            current_contract_hash.into(),
            assert_all_elements_are_the_same_stored_contract(contract_package_hash.into()),
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

    let calls = vec![
        Call {
            contract_address: ContractAddress::ContractPackageHash(contract_package_hash.into()),
            target_method: "forwarder_contract".to_string(),
            entry_point_type: EntryPointType::Contract
        };
        depth_limit
    ];

    let call_forwarder_request = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        contract_package_hash.into(),
        None,
        CONTRACT_FORWARDER_ENTRYPOINT_CONTRACT,
        runtime_args! {
            ARG_CALLS => calls,
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
            i + 2,
            current_contract_hash.into(),
            assert_all_elements_are_the_same_stored_contract(contract_package_hash.into()),
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

    let calls = vec![
        Call {
            contract_address: ContractAddress::ContractPackageHash(contract_package_hash.into()),
            target_method: CONTRACT_FORWARDER_ENTRYPOINT_CONTRACT.to_string(),
            entry_point_type: EntryPointType::Contract,
        };
        depth_limit
    ];

    let call_forwarder_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        CONTRACT_FORWARDER_ENTRYPOINT_CONTRACT,
        runtime_args! {
            ARG_CALLS => calls,
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
            i + 2,
            current_contract_hash.into(),
            assert_all_elements_are_the_same_stored_contract(contract_package_hash.into()),
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

    let calls = vec![
        Call {
            contract_address: ContractAddress::ContractPackageHash(contract_package_hash.into()),
            target_method: "forwarder_contract".to_string(),
            entry_point_type: EntryPointType::Contract,
        };
        depth_limit
    ];

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
        CONTRACT_FORWARDER_ENTRYPOINT_CONTRACT,
        runtime_args! {
            ARG_CALLS => calls,
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
            i + 2,
            current_contract_hash.into(),
            assert_all_elements_are_the_same_stored_contract(contract_package_hash.into()),
        );
    }
}

fn run_forwarder_versioned_contract_by_hash_from_session_code(depth_limit: usize) {
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

    let calls = vec![
        Call {
            contract_address: ContractAddress::ContractPackageHash(contract_package_hash.into()),
            target_method: "forwarder_contract".to_string(),
            entry_point_type: EntryPointType::Contract,
        };
        depth_limit
    ];

    let call_forwarder_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_CALL_RECURSIVE_SUBCALL,
        runtime_args! {
            ARG_CALLS => calls,
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
            i + 2,
            current_contract_hash.into(),
            assert_all_elements_are_the_same_stored_contract(contract_package_hash.into()),
        );
    }
}

#[ignore]
#[test]
fn should_run_forwarder_versioned_contract_by_name() {
    for depth_limit in &[1, 5, 10usize] {
        run_forwarder_versioned_contract_by_name(*depth_limit);
    }
}

#[ignore]
#[test]
fn should_run_forwarder_versioned_contract_by_hash() {
    for depth_limit in &[1, 5, 10usize] {
        run_forwarder_versioned_contract_by_hash(*depth_limit);
    }
}

#[ignore]
#[test]
fn should_run_forwarder_contract_by_name() {
    for depth_limit in &[1, 5, 10usize] {
        run_forwarder_contract_by_name(*depth_limit);
    }
}

#[ignore]
#[test]
fn should_run_forwarder_contract_by_hash() {
    for depth_limit in &[1, 5, 10usize] {
        run_forwarder_contract_by_hash(*depth_limit);
    }
}

#[ignore]
#[test]
fn should_run_forwarder_versioned_contract_by_hash_from_session_code() {
    for depth_limit in &[1, 5, 10usize] {
        run_forwarder_versioned_contract_by_hash_from_session_code(*depth_limit);
    }
}

#[allow(dead_code)]
fn run_forwarder_versioned_contract_by_name_as_payment(depth_limit: usize) {
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

    let calls = vec![
        Call {
            contract_address: ContractAddress::ContractPackageHash(contract_package_hash.into()),
            target_method: CONTRACT_FORWARDER_ENTRYPOINT_SESSION.to_string(),
            entry_point_type: EntryPointType::Contract,
        };
        depth_limit
    ];

    let execute_request = {
        let mut rng = rand::thread_rng();
        let deploy_hash = rng.gen();

        let sender = *DEFAULT_ACCOUNT_ADDR;

        let args = runtime_args! {
            ARG_CALLS => calls,
            ARG_CURRENT_DEPTH => 0u8,
        };

        let deploy = DeployItemBuilder::new()
            .with_address(sender)
            .with_stored_versioned_payment_contract_by_name(
                CONTRACT_PACKAGE_NAME,
                None,
                CONTRACT_FORWARDER_ENTRYPOINT_SESSION,
                args,
            )
            .with_session_bytes(wasm::do_nothing_bytes(), RuntimeArgs::default())
            .with_authorization_keys(&[sender])
            .with_deploy_hash(deploy_hash)
            .build();

        ExecuteRequestBuilder::new().push_deploy(deploy).build()
    };

    builder.exec(execute_request).commit().expect_success();

    let value = builder
        .query(None, Key::Hash(contract_package_hash), &[])
        .unwrap();

    let contract_package = match value {
        StoredValue::ContractPackage(package) => package,
        _ => panic!("unreachable"),
    };

    let _current_contract_hash = contract_package.current_contract_hash().unwrap();
    for i in 0..depth_limit {
        assert_expected(
            &mut builder,
            &format!("forwarder-{}", i),
            *DEFAULT_ACCOUNT_ADDR,
            i + 2,
            (*DEFAULT_ACCOUNT_ADDR).into(),
            assert_all_elements_are_the_same_stored_session(
                *DEFAULT_ACCOUNT_ADDR,
                contract_package_hash.into(),
            ),
        );
    }
}

#[ignore]
#[test]
fn should_run_forwarder_versioned_contract_by_name_as_payment() {
    // above a depth_limit of 5, we hit the gas limit
    for depth_limit in &[1, 5] {
        run_forwarder_versioned_contract_by_name_as_payment(*depth_limit);
    }
}

#[allow(dead_code)]
fn run_forwarder_versioned_contract_by_name_as_session(depth_limit: usize) {
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

    let calls = vec![
        Call {
            contract_address: ContractAddress::ContractPackageHash(contract_package_hash.into()),
            target_method: CONTRACT_FORWARDER_ENTRYPOINT_SESSION.to_string(),
            entry_point_type: EntryPointType::Contract,
        };
        depth_limit
    ];

    let call_forwarder_request = ExecuteRequestBuilder::versioned_contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_PACKAGE_NAME,
        None,
        CONTRACT_FORWARDER_ENTRYPOINT_SESSION,
        runtime_args! {
            ARG_CALLS => calls,
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

    let _current_contract_hash = contract_package.current_contract_hash().unwrap();
    for i in 0..depth_limit {
        assert_expected(
            &mut builder,
            &format!("forwarder-{}", i),
            *DEFAULT_ACCOUNT_ADDR,
            i + 2,
            (*DEFAULT_ACCOUNT_ADDR).into(),
            assert_all_elements_are_the_same_stored_session(
                *DEFAULT_ACCOUNT_ADDR,
                contract_package_hash.into(),
            ),
        );
    }
}

#[ignore]
#[test]
fn should_run_forwarder_versioned_contract_by_name_as_session() {
    for depth_limit in &[1, 5, 10] {
        run_forwarder_versioned_contract_by_name_as_session(*depth_limit);
    }
}
