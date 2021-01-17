use std::convert::TryFrom;

use casper_execution_engine::{shared, shared::stored_value::StoredValue};
use casper_types::{contracts::NamedKeys, PublicKey, URef};

use crate::{Error, Result};

/// An `Account` instance.
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Account {
    inner: shared::account::Account,
}

impl Account {
    /// creates a new Account instance.
    pub(crate) fn new(account: shared::account::Account) -> Self {
        Account { inner: account }
    }

    /// Returns the public_key.
    pub fn public_key(&self) -> PublicKey {
        self.inner.public_key()
    }

    /// Returns the named_keys.
    pub fn named_keys(&self) -> &NamedKeys {
        self.inner.named_keys()
    }

    /// Returns the main_purse.
    pub fn main_purse(&self) -> URef {
        self.inner.main_purse()
    }
}

impl From<shared::account::Account> for Account {
    fn from(value: shared::account::Account) -> Self {
        Account::new(value)
    }
}

impl TryFrom<StoredValue> for Account {
    type Error = Error;

    fn try_from(value: StoredValue) -> Result<Self> {
        match value {
            StoredValue::Account(account) => Ok(Account::new(account)),
            _ => Err(Error::from(String::from("StoredValue is not an Account"))),
        }
    }
}
