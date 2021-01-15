// TODO - remove once schemars stops causing warning.
#![allow(clippy::field_reassign_with_default)]

use datasize::DataSize;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use casper_execution_engine::shared::account::Account as ExecutionEngineAccount;
use casper_types::{NamedKey, PublicKey, URef};

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize, DataSize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct AssociatedKey {
    public_key: PublicKey,
    weight: u8,
}

/// Thresholds that have to be met when executing an action of a certain type.
#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize, DataSize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ActionThresholds {
    deployment: u8,
    key_management: u8,
}

/// Structure representing a user's account, stored in global state.
#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize, DataSize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Account {
    public_key: PublicKey,
    #[data_size(skip)]
    named_keys: Vec<NamedKey>,
    #[data_size(skip)]
    main_purse: URef,
    associated_keys: Vec<AssociatedKey>,
    action_thresholds: ActionThresholds,
}

impl From<&ExecutionEngineAccount> for Account {
    fn from(ee_account: &ExecutionEngineAccount) -> Self {
        Account {
            public_key: ee_account.public_key(),
            named_keys: ee_account
                .named_keys()
                .iter()
                .map(|(name, key)| NamedKey {
                    name: name.clone(),
                    key: key.to_formatted_string(),
                })
                .collect(),
            main_purse: ee_account.main_purse(),
            associated_keys: ee_account
                .associated_keys()
                .map(|(public_key, weight)| AssociatedKey {
                    public_key: *public_key,
                    weight: weight.value(),
                })
                .collect(),
            action_thresholds: ActionThresholds {
                deployment: ee_account.action_thresholds().deployment().value(),
                key_management: ee_account.action_thresholds().key_management().value(),
            },
        }
    }
}
