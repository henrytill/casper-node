//! Types used to allow creation of Wasm contracts and tests for use on the Casper Platform.
//!
//! # `no_std`
//!
//! By default, the library is `no_std`, however you can enable full `std` functionality by enabling
//! the crate's `std` feature.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(
    not(feature = "no-unstable-features"),
    feature(min_specialization, try_reserve)
)]
#![doc(html_root_url = "https://docs.rs/casper-types/0.5.0")]
#![doc(
    html_favicon_url = "https://raw.githubusercontent.com/CasperLabs/casper-node/master/images/CasperLabs_Logo_Favicon_RGB_50px.png",
    html_logo_url = "https://raw.githubusercontent.com/CasperLabs/casper-node/master/images/CasperLabs_Logo_Symbol_RGB.png",
    test(attr(forbid(warnings)))
)]
#![warn(missing_docs)]

extern crate alloc;
#[cfg(any(feature = "std", test))]
#[macro_use]
extern crate std;

pub use access_rights::{AccessRights, ACCESS_RIGHTS_SERIALIZED_LENGTH};
#[doc(inline)]
pub use api_error::ApiError;
pub use block_time::{BlockTime, BLOCKTIME_SERIALIZED_LENGTH};
pub use cl_type::{named_key_type, CLType, CLTyped};
pub use cl_value::{CLTypeMismatch, CLValue, CLValueError};
pub use contract_wasm::ContractWasm;
pub use contracts::{
    Contract, ContractPackage, ContractVersion, ContractVersionKey, EntryPoint, EntryPointAccess,
    EntryPointType, EntryPoints, Group, Parameter,
};
pub use crypto::*;
pub use deploy_info::DeployInfo;
pub use execution_result::{
    ExecutionEffect, ExecutionResult, OpKind, Operation, Transform, TransformEntry,
};
#[doc(inline)]
pub use key::{
    ContractHash, ContractPackageHash, ContractWasmHash, HashAddr, Key, BLAKE2B_DIGEST_LENGTH,
    KEY_HASH_LENGTH,
};
pub use named_key::NamedKey;
pub use phase::{Phase, PHASE_SERIALIZED_LENGTH};
pub use protocol_version::{ProtocolVersion, VersionCheckResult};
pub use runtime_args::{NamedArg, RuntimeArgs};
pub use semver::{SemVer, SEM_VER_SERIALIZED_LENGTH};
pub use system_contract_type::SystemContractType;
pub use transfer::{DeployHash, Transfer, TransferAddr, DEPLOY_HASH_LENGTH, TRANSFER_ADDR_LENGTH};
pub use transfer_result::{TransferResult, TransferredTo};
pub use uref::{FromStrError as URefFromStrError, URef, UREF_ADDR_LENGTH, UREF_SERIALIZED_LENGTH};

pub use crate::uint::{UIntParseError, U128, U256, U512};

mod access_rights;
pub mod account;
pub mod api_error;
pub mod auction;
mod block_time;
pub mod bytesrepr;
mod cl_type;
mod cl_value;
mod contract_wasm;
pub mod contracts;
pub mod crypto;
mod deploy_info;
mod execution_result;
#[cfg(any(feature = "gens", test))]
pub mod gens;
mod key;
pub mod mint;
mod named_key;
mod phase;
pub mod proof_of_stake;
mod protocol_version;
pub mod runtime_args;
mod semver;
pub mod standard_payment;
pub mod system_contract_errors;
pub mod system_contract_type;
mod transfer;
mod transfer_result;
mod uint;
mod uref;
