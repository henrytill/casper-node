use crate::{system_contract_errors::mint::Error, PublicKey, URef, U512};

/// Provides functionality of a system module.
pub trait SystemProvider {
    /// Records a transfer.
    fn record_transfer(
        &mut self,
        maybe_to: Option<PublicKey>,
        source: URef,
        target: URef,
        amount: U512,
        id: Option<u64>,
    ) -> Result<(), Error>;
}
