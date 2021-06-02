use casper_engine_test_support::{
    internal::{ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_RUN_GENESIS_REQUEST},
    DEFAULT_ACCOUNT_ADDR,
};
use casper_execution_engine::shared::stored_value::StoredValue;
use casper_types::{runtime_args, system::CallStackElement, CLValue, HashAddr, Key, RuntimeArgs};

const CONTRACT_RECURSIVE_SUBCALL: &str = "ee_1217_recursive_subcall.wasm";

const PACKAGE_NAME: &str = "forwarder";
const CONTRACT_FORWARDER_ENTRYPOINT: &str = "forwarder";

const ARG_TARGET_CONTRACT_HASH: &str = "target_contract_hash";
const ARG_TARGET_METHOD: &str = "target_method";
const ARG_LIMIT: &str = "limit";
const ARG_CURRENT_DEPTH: &str = "current_depth";

#[ignore]
#[test]
fn should_fail_to_call_auction_as_non_session_code() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST);

    // store
    {
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

    {
        let default_account = builder.get_account(*DEFAULT_ACCOUNT_ADDR).unwrap();

        println!("default_account: {:#?}", default_account);

        let contract_package_hash: HashAddr = default_account
            .named_keys()
            .get(PACKAGE_NAME)
            .cloned()
            .and_then(Key::into_hash)
            .unwrap();

        let call_forwarder_request =
            ExecuteRequestBuilder::versioned_contract_call_by_hash_key_name(
                *DEFAULT_ACCOUNT_ADDR,
                PACKAGE_NAME,
                None,
                CONTRACT_FORWARDER_ENTRYPOINT,
                runtime_args! {
                    ARG_TARGET_CONTRACT_HASH => contract_package_hash,
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
}
