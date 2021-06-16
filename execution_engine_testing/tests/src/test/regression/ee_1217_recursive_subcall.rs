use ee_1217_recursive_subcall::{Call, ContractAddress};
use num_traits::One;

use casper_engine_test_support::{
    internal::{
        ExecuteRequestBuilder, InMemoryWasmTestBuilder, WasmTestBuilder,
        DEFAULT_RUN_GENESIS_REQUEST,
    },
    DEFAULT_ACCOUNT_ADDR,
};
use casper_execution_engine::{
    shared::{account::Account, stored_value::StoredValue},
    storage::global_state::in_memory::InMemoryGlobalState,
};
use casper_types::{
    runtime_args, system::CallStackElement, CLValue, EntryPointType, HashAddr, Key, RuntimeArgs,
};

const CONTRACT_RECURSIVE_SUBCALL: &str = "ee_1217_recursive_subcall.wasm";
const CONTRACT_CALL_RECURSIVE_SUBCALL: &str = "ee_1217_call_recursive_subcall.wasm";

const CONTRACT_PACKAGE_NAME: &str = "forwarder";
const CONTRACT_NAME: &str = "our_contract_name";

const CONTRACT_FORWARDER_ENTRYPOINT_CONTRACT: &str = "forwarder_contract";
const CONTRACT_FORWARDER_ENTRYPOINT_SESSION: &str = "forwarder_session";

const ARG_CALLS: &str = "calls";
const ARG_CURRENT_DEPTH: &str = "current_depth";

fn store_contract(builder: &mut WasmTestBuilder<InMemoryGlobalState>, session_filename: &str) {
    let store_contract_request =
        ExecuteRequestBuilder::standard(*DEFAULT_ACCOUNT_ADDR, session_filename, runtime_args! {})
            .build();
    builder
        .exec(store_contract_request)
        .commit()
        .expect_success();
}

trait AccountExt {
    fn get_hash(&self, key: &str) -> HashAddr;
}

impl AccountExt for Account {
    fn get_hash(&self, key: &str) -> HashAddr {
        self.named_keys()
            .get(key)
            .cloned()
            .and_then(Key::into_hash)
            .unwrap()
    }
}

trait BuilderExt {
    fn get_call_stack_from_session_context(
        &mut self,
        stored_call_stack_key: &str,
    ) -> Vec<CallStackElement>;
    fn get_call_stack_from_contract_context(
        &mut self,
        stored_call_stack_key: &str,
        contract_package_hash: HashAddr,
    ) -> Vec<CallStackElement>;
}

impl BuilderExt for WasmTestBuilder<InMemoryGlobalState> {
    fn get_call_stack_from_session_context(
        &mut self,
        stored_call_stack_key: &str,
    ) -> Vec<CallStackElement> {
        let cl_value = self
            .query(
                None,
                (*DEFAULT_ACCOUNT_ADDR).into(),
                &[stored_call_stack_key.to_string()],
            )
            .unwrap();

        cl_value
            .as_cl_value()
            .cloned()
            .map(CLValue::into_t::<Vec<CallStackElement>>)
            .unwrap()
            .unwrap()
    }

    fn get_call_stack_from_contract_context(
        &mut self,
        stored_call_stack_key: &str,
        contract_package_hash: HashAddr,
    ) -> Vec<CallStackElement> {
        let value = self
            .query(None, Key::Hash(contract_package_hash), &[])
            .unwrap();

        let contract_package = match value {
            StoredValue::ContractPackage(package) => package,
            _ => panic!("unreachable"),
        };

        let current_contract_hash = contract_package.current_contract_hash().unwrap();

        let cl_value = self
            .query(
                None,
                current_contract_hash.into(),
                &[stored_call_stack_key.to_string()],
            )
            .unwrap();

        cl_value
            .as_cl_value()
            .cloned()
            .map(CLValue::into_t::<Vec<CallStackElement>>)
            .unwrap()
            .unwrap()
    }
}

fn setup() -> WasmTestBuilder<InMemoryGlobalState> {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST);
    store_contract(&mut builder, CONTRACT_RECURSIVE_SUBCALL);
    builder
}

fn assert_each_context_has_correct_call_stack_info(
    builder: &mut InMemoryWasmTestBuilder,
    calls: &[Call],
    current_contract_package_hash: HashAddr,
) {
    // query for and verify that all the elements in the call stack match their
    // pre-defined Call element
    for (i, call) in calls.iter().enumerate() {
        let stored_call_stack_key = format!("forwarder-{}", i);
        // we need to know where to look for the call stack information
        let call_stack = match call.entry_point_type {
            EntryPointType::Contract => builder.get_call_stack_from_contract_context(
                &stored_call_stack_key,
                current_contract_package_hash,
            ),
            EntryPointType::Session => {
                builder.get_call_stack_from_session_context(&stored_call_stack_key)
            }
        };
        assert_eq!(
            call_stack.len(),
            i + 2,
            "call stack len was an unexpected size {}, should be {} {:#?}",
            call_stack.len(),
            i + 2,
            call_stack,
        );
        assert_call_stack_matches_calls(call_stack, &calls);
    }
}

fn assert_call_stack_matches_calls(call_stack: Vec<CallStackElement>, calls: &[Call]) {
    let (head, rest) = call_stack.split_at(usize::one());

    assert_eq!(
        head,
        [CallStackElement::Session {
            account_hash: *DEFAULT_ACCOUNT_ADDR,
        }],
    );
    for (index, expected_call_stack_element) in rest.iter().enumerate() {
        let maybe_call = calls.get(index);
        match (maybe_call, expected_call_stack_element) {
            // Versioned Call with EntryPointType::Contract
            (
                Some(Call {
                    entry_point_type,
                    contract_address:
                        ContractAddress::ContractPackageHash(current_contract_package_hash),
                    ..
                }),
                CallStackElement::StoredContract {
                    contract_package_hash,
                    ..
                },
            ) if *entry_point_type == EntryPointType::Contract
                && *contract_package_hash == *current_contract_package_hash => {}

            // Unversioned Call with EntryPointType::Contract
            (
                Some(Call {
                    entry_point_type,
                    contract_address: ContractAddress::ContractHash(current_contract_hash),
                    ..
                }),
                CallStackElement::StoredContract { contract_hash, .. },
            ) if *entry_point_type == EntryPointType::Contract
                && *contract_hash == *current_contract_hash => {}

            // Versioned Call with EntryPointType::Session
            (
                Some(Call {
                    entry_point_type,
                    contract_address:
                        ContractAddress::ContractPackageHash(current_contract_package_hash),
                    ..
                }),
                CallStackElement::StoredSession {
                    account_hash,
                    contract_package_hash,
                    ..
                },
            ) if *entry_point_type == EntryPointType::Session
                && *account_hash == *DEFAULT_ACCOUNT_ADDR
                && *contract_package_hash == *current_contract_package_hash => {}

            // Unversioned Call with EntryPointType::Session
            (
                Some(Call {
                    entry_point_type,
                    contract_address: ContractAddress::ContractHash(current_contract_hash),
                    ..
                }),
                CallStackElement::StoredSession {
                    account_hash,
                    contract_hash,
                    ..
                },
            ) if *entry_point_type == EntryPointType::Session
                && *account_hash == *DEFAULT_ACCOUNT_ADDR
                && *contract_hash == *current_contract_hash => {}

            _ => assert!(
                false,
                "call stack element {:#?} didn't match expected call {:#?} at index {}, {:#?}",
                expected_call_stack_element, maybe_call, index, rest,
            ),
        }
    }
}

mod session {
    use casper_engine_test_support::{internal::ExecuteRequestBuilder, DEFAULT_ACCOUNT_ADDR};
    use casper_types::{runtime_args, EntryPointType, RuntimeArgs};
    use ee_1217_recursive_subcall::{Call, ContractAddress};

    use super::{
        AccountExt, ARG_CALLS, ARG_CURRENT_DEPTH, CONTRACT_CALL_RECURSIVE_SUBCALL,
        CONTRACT_FORWARDER_ENTRYPOINT_CONTRACT, CONTRACT_FORWARDER_ENTRYPOINT_SESSION,
        CONTRACT_NAME, CONTRACT_PACKAGE_NAME,
    };

    #[ignore]
    #[test]
    fn should_run_session_bytes_to_stored_versioned_contract() {
        for len in &[1, 5, 10] {
            let mut builder = super::setup();
            let default_account = builder.get_account(*DEFAULT_ACCOUNT_ADDR).unwrap();
            let current_contract_package_hash = default_account.get_hash(CONTRACT_PACKAGE_NAME);

            let calls = vec![
                Call {
                    contract_address: ContractAddress::ContractPackageHash(
                        current_contract_package_hash.into()
                    ),
                    target_method: CONTRACT_FORWARDER_ENTRYPOINT_CONTRACT.to_string(),
                    entry_point_type: EntryPointType::Contract,
                };
                *len
            ];
            let execute_request = ExecuteRequestBuilder::standard(
                *DEFAULT_ACCOUNT_ADDR,
                CONTRACT_CALL_RECURSIVE_SUBCALL,
                runtime_args! {
                    ARG_CALLS => calls.clone(),
                    ARG_CURRENT_DEPTH => 0u8,
                },
            )
            .build();

            builder.exec(execute_request).commit().expect_success();

            super::assert_each_context_has_correct_call_stack_info(
                &mut builder,
                &calls,
                current_contract_package_hash,
            );
        }
    }

    #[ignore]
    #[test]
    fn should_run_stored_versioned_contract_by_name_to_stored_versioned_contract() {
        for len in &[1, 5, 10] {
            let mut builder = super::setup();
            let default_account = builder.get_account(*DEFAULT_ACCOUNT_ADDR).unwrap();
            let current_contract_package_hash = default_account.get_hash(CONTRACT_PACKAGE_NAME);

            let calls = vec![
                Call {
                    contract_address: ContractAddress::ContractPackageHash(
                        current_contract_package_hash.into()
                    ),
                    target_method: CONTRACT_FORWARDER_ENTRYPOINT_CONTRACT.to_string(),
                    entry_point_type: EntryPointType::Contract,
                };
                *len
            ];
            let execute_request = ExecuteRequestBuilder::versioned_contract_call_by_name(
                *DEFAULT_ACCOUNT_ADDR,
                CONTRACT_PACKAGE_NAME,
                None,
                CONTRACT_FORWARDER_ENTRYPOINT_CONTRACT,
                runtime_args! {
                    ARG_CALLS => calls.clone(),
                    ARG_CURRENT_DEPTH => 0u8,
                },
            )
            .build();

            builder.exec(execute_request).commit().expect_success();

            super::assert_each_context_has_correct_call_stack_info(
                &mut builder,
                &calls,
                current_contract_package_hash,
            );
        }
    }

    #[ignore]
    #[test]
    fn should_run_stored_versioned_contract_by_hash_to_stored_versioned_contract() {
        for len in &[1, 5, 10] {
            let mut builder = super::setup();
            let default_account = builder.get_account(*DEFAULT_ACCOUNT_ADDR).unwrap();
            let current_contract_package_hash = default_account.get_hash(CONTRACT_PACKAGE_NAME);

            let calls = vec![
                Call {
                    contract_address: ContractAddress::ContractPackageHash(
                        current_contract_package_hash.into()
                    ),
                    target_method: CONTRACT_FORWARDER_ENTRYPOINT_CONTRACT.to_string(),
                    entry_point_type: EntryPointType::Contract,
                };
                *len
            ];
            let execute_request = ExecuteRequestBuilder::versioned_contract_call_by_hash(
                *DEFAULT_ACCOUNT_ADDR,
                current_contract_package_hash.into(),
                None,
                CONTRACT_FORWARDER_ENTRYPOINT_CONTRACT,
                runtime_args! {
                    ARG_CALLS => calls.clone(),
                    ARG_CURRENT_DEPTH => 0u8,
                },
            )
            .build();

            builder.exec(execute_request).commit().expect_success();

            super::assert_each_context_has_correct_call_stack_info(
                &mut builder,
                &calls,
                current_contract_package_hash,
            );
        }
    }

    #[ignore]
    #[test]
    fn should_run_stored_contract_by_name_to_stored_versioned_contract() {
        for len in &[1, 5, 10] {
            let mut builder = super::setup();
            let default_account = builder.get_account(*DEFAULT_ACCOUNT_ADDR).unwrap();
            let current_contract_package_hash = default_account.get_hash(CONTRACT_PACKAGE_NAME);

            let calls = vec![
                Call {
                    contract_address: ContractAddress::ContractPackageHash(
                        current_contract_package_hash.into()
                    ),
                    target_method: CONTRACT_FORWARDER_ENTRYPOINT_CONTRACT.to_string(),
                    entry_point_type: EntryPointType::Contract,
                };
                *len
            ];
            let execute_request = ExecuteRequestBuilder::contract_call_by_name(
                *DEFAULT_ACCOUNT_ADDR,
                CONTRACT_NAME,
                CONTRACT_FORWARDER_ENTRYPOINT_CONTRACT,
                runtime_args! {
                    ARG_CALLS => calls.clone(),
                    ARG_CURRENT_DEPTH => 0u8,
                },
            )
            .build();

            builder.exec(execute_request).commit().expect_success();

            super::assert_each_context_has_correct_call_stack_info(
                &mut builder,
                &calls,
                current_contract_package_hash,
            );
        }
    }

    #[ignore]
    #[test]
    fn should_run_stored_contract_by_hash_to_stored_versioned_contract() {
        for len in &[1, 5, 10] {
            let mut builder = super::setup();
            let default_account = builder.get_account(*DEFAULT_ACCOUNT_ADDR).unwrap();
            let current_contract_package_hash = default_account.get_hash(CONTRACT_PACKAGE_NAME);
            let current_contract_hash = default_account.get_hash(CONTRACT_NAME);

            let calls = vec![
                Call {
                    contract_address: ContractAddress::ContractPackageHash(
                        current_contract_package_hash.into()
                    ),
                    target_method: CONTRACT_FORWARDER_ENTRYPOINT_CONTRACT.to_string(),
                    entry_point_type: EntryPointType::Contract,
                };
                *len
            ];
            let execute_request = ExecuteRequestBuilder::contract_call_by_hash(
                *DEFAULT_ACCOUNT_ADDR,
                current_contract_hash.into(),
                CONTRACT_FORWARDER_ENTRYPOINT_CONTRACT,
                runtime_args! {
                    ARG_CALLS => calls.clone(),
                    ARG_CURRENT_DEPTH => 0u8,
                },
            )
            .build();

            builder.exec(execute_request).commit().expect_success();

            super::assert_each_context_has_correct_call_stack_info(
                &mut builder,
                &calls,
                current_contract_package_hash,
            );
        }
    }

    #[ignore]
    #[test]
    fn should_run_stored_versioned_session_by_name_to_stored_versioned_session() {
        for len in &[1, 5, 10] {
            let mut builder = super::setup();
            let default_account = builder.get_account(*DEFAULT_ACCOUNT_ADDR).unwrap();
            let current_contract_package_hash = default_account.get_hash(CONTRACT_PACKAGE_NAME);

            let calls = vec![
                Call {
                    contract_address: ContractAddress::ContractPackageHash(
                        current_contract_package_hash.into()
                    ),
                    target_method: CONTRACT_FORWARDER_ENTRYPOINT_SESSION.to_string(),
                    entry_point_type: EntryPointType::Session,
                };
                *len
            ];
            let execute_request = ExecuteRequestBuilder::versioned_contract_call_by_name(
                *DEFAULT_ACCOUNT_ADDR,
                CONTRACT_PACKAGE_NAME,
                None,
                CONTRACT_FORWARDER_ENTRYPOINT_SESSION,
                runtime_args! {
                    ARG_CALLS => calls.clone(),
                    ARG_CURRENT_DEPTH => 0u8,
                },
            )
            .build();

            builder.exec(execute_request).commit().expect_success();

            super::assert_each_context_has_correct_call_stack_info(
                &mut builder,
                &calls,
                current_contract_package_hash,
            );
        }
    }

    #[ignore]
    #[test]
    fn should_run_stored_versioned_session_by_hash_to_stored_versioned_session() {
        for len in &[1, 5, 10] {
            let mut builder = super::setup();
            let default_account = builder.get_account(*DEFAULT_ACCOUNT_ADDR).unwrap();
            let current_contract_package_hash = default_account.get_hash(CONTRACT_PACKAGE_NAME);

            let calls = vec![
                Call {
                    contract_address: ContractAddress::ContractPackageHash(
                        current_contract_package_hash.into()
                    ),
                    target_method: CONTRACT_FORWARDER_ENTRYPOINT_SESSION.to_string(),
                    entry_point_type: EntryPointType::Session,
                };
                *len
            ];
            let execute_request = ExecuteRequestBuilder::versioned_contract_call_by_hash(
                *DEFAULT_ACCOUNT_ADDR,
                current_contract_package_hash.into(),
                None,
                CONTRACT_FORWARDER_ENTRYPOINT_SESSION,
                runtime_args! {
                    ARG_CALLS => calls.clone(),
                    ARG_CURRENT_DEPTH => 0u8,
                },
            )
            .build();

            builder.exec(execute_request).commit().expect_success();

            super::assert_each_context_has_correct_call_stack_info(
                &mut builder,
                &calls,
                current_contract_package_hash,
            );
        }
    }
}

mod payment {
    use rand::Rng;

    use casper_engine_test_support::{
        internal::{DeployItemBuilder, ExecuteRequestBuilder},
        DEFAULT_ACCOUNT_ADDR,
    };
    use casper_execution_engine::shared::wasm;
    use casper_types::{runtime_args, EntryPointType, RuntimeArgs};
    use ee_1217_recursive_subcall::{Call, ContractAddress};

    use super::{
        AccountExt, ARG_CALLS, ARG_CURRENT_DEPTH, CONTRACT_FORWARDER_ENTRYPOINT_CONTRACT,
        CONTRACT_FORWARDER_ENTRYPOINT_SESSION, CONTRACT_NAME, CONTRACT_PACKAGE_NAME,
    };

    #[ignore]
    #[test]
    fn stored_versioned_session_by_name_to_stored_versioned_session() {
        // going further than 5 will git the gas limit
        for len in &[1, 5] {
            let mut builder = super::setup();
            let default_account = builder.get_account(*DEFAULT_ACCOUNT_ADDR).unwrap();
            let current_contract_package_hash = default_account.get_hash(CONTRACT_PACKAGE_NAME);

            let calls = vec![
                Call {
                    contract_address: ContractAddress::ContractPackageHash(
                        current_contract_package_hash.into()
                    ),
                    target_method: CONTRACT_FORWARDER_ENTRYPOINT_SESSION.to_string(),
                    entry_point_type: EntryPointType::Session,
                };
                *len
            ];

            let execute_request = {
                let mut rng = rand::thread_rng();
                let deploy_hash = rng.gen();

                let sender = *DEFAULT_ACCOUNT_ADDR;

                let args = runtime_args! {
                    ARG_CALLS => calls.clone(),
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

            super::assert_each_context_has_correct_call_stack_info(
                &mut builder,
                &calls,
                current_contract_package_hash,
            );
        }
    }

    #[ignore]
    #[test]
    fn stored_versioned_session_by_hash_to_stored_versioned_session() {
        // going further than 5 will git the gas limit
        for len in &[1, 5] {
            let mut builder = super::setup();
            let default_account = builder.get_account(*DEFAULT_ACCOUNT_ADDR).unwrap();
            let current_contract_package_hash = default_account.get_hash(CONTRACT_PACKAGE_NAME);

            let calls = vec![
                Call {
                    contract_address: ContractAddress::ContractPackageHash(
                        current_contract_package_hash.into()
                    ),
                    target_method: CONTRACT_FORWARDER_ENTRYPOINT_SESSION.to_string(),
                    entry_point_type: EntryPointType::Session,
                };
                *len
            ];

            let execute_request = {
                let mut rng = rand::thread_rng();
                let deploy_hash = rng.gen();

                let sender = *DEFAULT_ACCOUNT_ADDR;

                let args = runtime_args! {
                    ARG_CALLS => calls.clone(),
                    ARG_CURRENT_DEPTH => 0u8,
                };

                let deploy = DeployItemBuilder::new()
                    .with_address(sender)
                    .with_stored_versioned_payment_contract_by_hash(
                        current_contract_package_hash.into(),
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

            super::assert_each_context_has_correct_call_stack_info(
                &mut builder,
                &calls,
                current_contract_package_hash,
            );
        }
    }

    #[ignore]
    #[test]
    fn stored_versioned_session_by_name_to_stored_session() {
        // going further than 5 will git the gas limit
        for len in &[1, 5] {
            let mut builder = super::setup();
            let default_account = builder.get_account(*DEFAULT_ACCOUNT_ADDR).unwrap();
            let current_contract_package_hash = default_account.get_hash(CONTRACT_PACKAGE_NAME);
            let current_contract_hash = default_account.get_hash(CONTRACT_NAME);

            let calls = vec![
                Call {
                    contract_address: ContractAddress::ContractHash(current_contract_hash.into()),
                    target_method: CONTRACT_FORWARDER_ENTRYPOINT_SESSION.to_string(),
                    entry_point_type: EntryPointType::Session,
                };
                *len
            ];

            let execute_request = {
                let mut rng = rand::thread_rng();
                let deploy_hash = rng.gen();

                let sender = *DEFAULT_ACCOUNT_ADDR;

                let args = runtime_args! {
                    ARG_CALLS => calls.clone(),
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

            super::assert_each_context_has_correct_call_stack_info(
                &mut builder,
                &calls,
                current_contract_package_hash,
            );
        }
    }

    #[ignore]
    #[test]
    fn stored_versioned_session_by_hash_to_stored_session() {
        // going further than 5 will git the gas limit
        for len in &[1, 5] {
            let mut builder = super::setup();
            let default_account = builder.get_account(*DEFAULT_ACCOUNT_ADDR).unwrap();
            let current_contract_package_hash = default_account.get_hash(CONTRACT_PACKAGE_NAME);
            let current_contract_hash = default_account.get_hash(CONTRACT_NAME);

            let calls = vec![
                Call {
                    contract_address: ContractAddress::ContractHash(current_contract_hash.into()),
                    target_method: CONTRACT_FORWARDER_ENTRYPOINT_SESSION.to_string(),
                    entry_point_type: EntryPointType::Session,
                };
                *len
            ];

            let execute_request = {
                let mut rng = rand::thread_rng();
                let deploy_hash = rng.gen();

                let sender = *DEFAULT_ACCOUNT_ADDR;

                let args = runtime_args! {
                    ARG_CALLS => calls.clone(),
                    ARG_CURRENT_DEPTH => 0u8,
                };

                let deploy = DeployItemBuilder::new()
                    .with_address(sender)
                    .with_stored_versioned_payment_contract_by_hash(
                        current_contract_package_hash.into(),
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

            super::assert_each_context_has_correct_call_stack_info(
                &mut builder,
                &calls,
                current_contract_package_hash,
            );
        }
    }

    #[ignore]
    #[test]
    fn stored_session_by_name_to_stored_versioned_session() {
        // going further than 5 will git the gas limit
        for len in &[1, 5] {
            let mut builder = super::setup();
            let default_account = builder.get_account(*DEFAULT_ACCOUNT_ADDR).unwrap();
            let current_contract_package_hash = default_account.get_hash(CONTRACT_PACKAGE_NAME);

            let calls = vec![
                Call {
                    contract_address: ContractAddress::ContractPackageHash(
                        current_contract_package_hash.into()
                    ),
                    target_method: CONTRACT_FORWARDER_ENTRYPOINT_SESSION.to_string(),
                    entry_point_type: EntryPointType::Session,
                };
                *len
            ];

            let execute_request = {
                let mut rng = rand::thread_rng();
                let deploy_hash = rng.gen();

                let sender = *DEFAULT_ACCOUNT_ADDR;

                let args = runtime_args! {
                    ARG_CALLS => calls.clone(),
                    ARG_CURRENT_DEPTH => 0u8,
                };

                let deploy = DeployItemBuilder::new()
                    .with_address(sender)
                    .with_stored_payment_named_key(
                        CONTRACT_NAME,
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

            super::assert_each_context_has_correct_call_stack_info(
                &mut builder,
                &calls,
                current_contract_package_hash,
            );
        }
    }

    #[ignore]
    #[test]
    fn stored_session_by_hash_to_stored_versioned_session() {
        // going further than 5 will git the gas limit
        for len in &[1, 5] {
            let mut builder = super::setup();
            let default_account = builder.get_account(*DEFAULT_ACCOUNT_ADDR).unwrap();
            let current_contract_package_hash = default_account.get_hash(CONTRACT_PACKAGE_NAME);
            let current_contract_hash = default_account.get_hash(CONTRACT_NAME);

            let calls = vec![
                Call {
                    contract_address: ContractAddress::ContractPackageHash(
                        current_contract_package_hash.into()
                    ),
                    target_method: CONTRACT_FORWARDER_ENTRYPOINT_SESSION.to_string(),
                    entry_point_type: EntryPointType::Session,
                };
                *len
            ];

            let execute_request = {
                let mut rng = rand::thread_rng();
                let deploy_hash = rng.gen();

                let sender = *DEFAULT_ACCOUNT_ADDR;

                let args = runtime_args! {
                    ARG_CALLS => calls.clone(),
                    ARG_CURRENT_DEPTH => 0u8,
                };

                let deploy = DeployItemBuilder::new()
                    .with_address(sender)
                    .with_stored_payment_hash(
                        current_contract_hash.into(),
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

            super::assert_each_context_has_correct_call_stack_info(
                &mut builder,
                &calls,
                current_contract_package_hash,
            );
        }
    }

    #[ignore]
    #[test]
    fn stored_session_by_name_to_stored_session() {
        // going further than 5 will git the gas limit
        for len in &[1, 5] {
            let mut builder = super::setup();
            let default_account = builder.get_account(*DEFAULT_ACCOUNT_ADDR).unwrap();
            let current_contract_package_hash = default_account.get_hash(CONTRACT_PACKAGE_NAME);
            let current_contract_hash = default_account.get_hash(CONTRACT_NAME);

            let calls = vec![
                Call {
                    contract_address: ContractAddress::ContractHash(current_contract_hash.into()),
                    target_method: CONTRACT_FORWARDER_ENTRYPOINT_SESSION.to_string(),
                    entry_point_type: EntryPointType::Session,
                };
                *len
            ];

            let execute_request = {
                let mut rng = rand::thread_rng();
                let deploy_hash = rng.gen();

                let sender = *DEFAULT_ACCOUNT_ADDR;

                let args = runtime_args! {
                    ARG_CALLS => calls.clone(),
                    ARG_CURRENT_DEPTH => 0u8,
                };

                let deploy = DeployItemBuilder::new()
                    .with_address(sender)
                    .with_stored_payment_named_key(
                        CONTRACT_NAME,
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

            super::assert_each_context_has_correct_call_stack_info(
                &mut builder,
                &calls,
                current_contract_package_hash,
            );
        }
    }

    #[ignore]
    #[test]
    fn stored_session_by_hash_to_stored_session() {
        // going further than 5 will git the gas limit
        for len in &[1, 5] {
            let mut builder = super::setup();
            let default_account = builder.get_account(*DEFAULT_ACCOUNT_ADDR).unwrap();
            let current_contract_package_hash = default_account.get_hash(CONTRACT_PACKAGE_NAME);
            let current_contract_hash = default_account.get_hash(CONTRACT_NAME);

            let calls = vec![
                Call {
                    contract_address: ContractAddress::ContractHash(current_contract_hash.into()),
                    target_method: CONTRACT_FORWARDER_ENTRYPOINT_SESSION.to_string(),
                    entry_point_type: EntryPointType::Session,
                };
                *len
            ];

            let execute_request = {
                let mut rng = rand::thread_rng();
                let deploy_hash = rng.gen();

                let sender = *DEFAULT_ACCOUNT_ADDR;

                let args = runtime_args! {
                    ARG_CALLS => calls.clone(),
                    ARG_CURRENT_DEPTH => 0u8,
                };

                let deploy = DeployItemBuilder::new()
                    .with_address(sender)
                    .with_stored_payment_hash(
                        current_contract_hash.into(),
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

            super::assert_each_context_has_correct_call_stack_info(
                &mut builder,
                &calls,
                current_contract_package_hash,
            );
        }
    }

    #[ignore]
    #[test]
    fn stored_session_by_name_to_stored_versioned_contract() {
        // going further than 5 will git the gas limit
        for len in &[1, 5] {
            let mut builder = super::setup();
            let default_account = builder.get_account(*DEFAULT_ACCOUNT_ADDR).unwrap();
            let current_contract_package_hash = default_account.get_hash(CONTRACT_PACKAGE_NAME);

            let calls = vec![
                Call {
                    contract_address: ContractAddress::ContractPackageHash(
                        current_contract_package_hash.into()
                    ),
                    target_method: CONTRACT_FORWARDER_ENTRYPOINT_CONTRACT.to_string(),
                    entry_point_type: EntryPointType::Contract,
                };
                *len
            ];

            let execute_request = {
                let mut rng = rand::thread_rng();
                let deploy_hash = rng.gen();

                let sender = *DEFAULT_ACCOUNT_ADDR;

                let args = runtime_args! {
                    ARG_CALLS => calls.clone(),
                    ARG_CURRENT_DEPTH => 0u8,
                };

                let deploy = DeployItemBuilder::new()
                    .with_address(sender)
                    .with_stored_payment_named_key(
                        CONTRACT_NAME,
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

            super::assert_each_context_has_correct_call_stack_info(
                &mut builder,
                &calls,
                current_contract_package_hash,
            );
        }
    }
    /*
    #[ignore]
    #[test]
    fn stored_session_by_hash_to_stored_versioned_contract() {
        // going further than 5 will git the gas limit
        for len in &[1, 5] {
            let mut builder = super::setup();
            let default_account = builder.get_account(*DEFAULT_ACCOUNT_ADDR).unwrap();
            let current_contract_package_hash = default_account.get_hash(CONTRACT_PACKAGE_NAME);
            let current_contract_hash = default_account.get_hash(CONTRACT_NAME);

            let calls = vec![
                Call {
                    contract_address: ContractAddress::ContractPackageHash(
                        current_contract_package_hash.into()
                    ),
                    target_method: CONTRACT_FORWARDER_ENTRYPOINT_CONTRACT.to_string(),
                    entry_point_type: EntryPointType::Contract,
                };
                *len
            ];

            let execute_request = {
                let mut rng = rand::thread_rng();
                let deploy_hash = rng.gen();

                let sender = *DEFAULT_ACCOUNT_ADDR;

                let args = runtime_args! {
                    ARG_CALLS => calls.clone(),
                    ARG_CURRENT_DEPTH => 0u8,
                };

                let deploy = DeployItemBuilder::new()
                    .with_address(sender)
                    .with_stored_payment_hash(
                        current_contract_hash.into(),
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

            super::assert_each_context_has_correct_call_stack_info(
                &mut builder,
                &calls,
                current_contract_package_hash,
            );
        }
    }

    #[ignore]
    #[test]
    fn stored_session_by_name_to_stored_contract() {
        // going further than 5 will git the gas limit
        for len in &[1, 5] {
            let mut builder = super::setup();
            let default_account = builder.get_account(*DEFAULT_ACCOUNT_ADDR).unwrap();
            let current_contract_package_hash = default_account.get_hash(CONTRACT_PACKAGE_NAME);
            let current_contract_hash = default_account.get_hash(CONTRACT_NAME);

            let calls = vec![
                Call {
                    contract_address: ContractAddress::ContractHash(current_contract_hash.into()),
                    target_method: CONTRACT_FORWARDER_ENTRYPOINT_CONTRACT.to_string(),
                    entry_point_type: EntryPointType::Contract,
                };
                *len
            ];

            let execute_request = {
                let mut rng = rand::thread_rng();
                let deploy_hash = rng.gen();

                let sender = *DEFAULT_ACCOUNT_ADDR;

                let args = runtime_args! {
                    ARG_CALLS => calls.clone(),
                    ARG_CURRENT_DEPTH => 0u8,
                };

                let deploy = DeployItemBuilder::new()
                    .with_address(sender)
                    .with_stored_payment_named_key(
                        CONTRACT_NAME,
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

            super::assert_each_context_has_correct_call_stack_info(
                &mut builder,
                &calls,
                current_contract_package_hash,
            );
        }
    }

    #[ignore]
    #[test]
    fn stored_session_by_hash_to_stored_contract() {
        // going further than 5 will git the gas limit
        for len in &[1, 5] {
            let mut builder = super::setup();
            let default_account = builder.get_account(*DEFAULT_ACCOUNT_ADDR).unwrap();
            let current_contract_package_hash = default_account.get_hash(CONTRACT_PACKAGE_NAME);
            let current_contract_hash = default_account.get_hash(CONTRACT_NAME);

            let calls = vec![
                Call {
                    contract_address: ContractAddress::ContractHash(current_contract_hash.into()),
                    target_method: CONTRACT_FORWARDER_ENTRYPOINT_CONTRACT.to_string(),
                    entry_point_type: EntryPointType::Contract,
                };
                *len
            ];

            let execute_request = {
                let mut rng = rand::thread_rng();
                let deploy_hash = rng.gen();

                let sender = *DEFAULT_ACCOUNT_ADDR;

                let args = runtime_args! {
                    ARG_CALLS => calls.clone(),
                    ARG_CURRENT_DEPTH => 0u8,
                };

                let deploy = DeployItemBuilder::new()
                    .with_address(sender)
                    .with_stored_payment_hash(
                        current_contract_hash.into(),
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

            super::assert_each_context_has_correct_call_stack_info(
                &mut builder,
                &calls,
                current_contract_package_hash,
            );
        }
    }
    */
}
