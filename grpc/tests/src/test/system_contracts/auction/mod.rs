mod bids;
mod distribute;

use casper_engine_test_support::internal::{ExecuteRequestBuilder, InMemoryWasmTestBuilder};
use casper_types::{self, auction::METHOD_RUN_AUCTION, runtime_args, RuntimeArgs, SYSTEM_ACCOUNT};

const ARG_ENTRY_POINT: &str = "entry_point";
const CONTRACT_AUCTION_BIDS: &str = "auction_bids.wasm";

fn make_run_auction_request() -> ExecuteRequestBuilder {
    ExecuteRequestBuilder::standard(
        SYSTEM_ACCOUNT,
        CONTRACT_AUCTION_BIDS,
        runtime_args! {
            ARG_ENTRY_POINT => METHOD_RUN_AUCTION
        },
    )
}

fn run_auction(builder: &mut InMemoryWasmTestBuilder) {
    let run_request = make_run_auction_request().build();
    builder.exec(run_request).commit().expect_success();
}
