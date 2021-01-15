//! Some functions to use in tests.

use casper_types::{contracts::NamedKeys, AccessRights, Key, PublicKey, URef};

use crate::shared::{account::Account, stored_value::StoredValue};

/// Returns an account value paired with its key
pub fn mocked_account(public_key: PublicKey) -> Vec<(Key, StoredValue)> {
    let purse = URef::new([0u8; 32], AccessRights::READ_ADD_WRITE);
    let account = Account::create(public_key, NamedKeys::new(), purse);
    vec![(Key::Account(public_key), StoredValue::Account(account))]
}
