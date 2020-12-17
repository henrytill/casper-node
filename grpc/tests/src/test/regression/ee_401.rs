use casper_engine_test_support::{
    internal::{ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_RUN_GENESIS_REQUEST},
    DEFAULT_ACCOUNT_PUBLIC_KEY,
};
use casper_types::RuntimeArgs;

const CONTRACT_EE_401_REGRESSION: &str = "ee_401_regression.wasm";
const CONTRACT_EE_401_REGRESSION_CALL: &str = "ee_401_regression_call.wasm";

#[ignore]
#[test]
fn should_execute_contracts_which_provide_extra_urefs() {
    let exec_request_1 = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_PUBLIC_KEY,
        CONTRACT_EE_401_REGRESSION,
        RuntimeArgs::default(),
    )
    .build();

    let exec_request_2 = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_PUBLIC_KEY,
        CONTRACT_EE_401_REGRESSION_CALL,
        RuntimeArgs::default(),
    )
    .build();

    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST);

    builder.exec(exec_request_1).expect_success().commit();
    builder.exec(exec_request_2).expect_success().commit();
}
