use casper_execution_engine::{
    core::engine_state::GetEraValidatorsRequest, shared::newtypes::Blake2bHash,
};
use casper_types::{auction, ProtocolVersion};

use crate::components::consensus;

/// Request for validator weights for a specific era.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatorWeightsByEraIdRequest {
    state_hash: Blake2bHash,
    era_id: consensus::EraId,
    protocol_version: ProtocolVersion,
}

impl ValidatorWeightsByEraIdRequest {
    /// Constructs a new ValidatorWeightsByEraIdRequest.
    pub fn new(
        state_hash: Blake2bHash,
        era_id: consensus::EraId,
        protocol_version: ProtocolVersion,
    ) -> Self {
        ValidatorWeightsByEraIdRequest {
            state_hash,
            era_id,
            protocol_version,
        }
    }

    /// Get the state hash.
    pub fn state_hash(&self) -> Blake2bHash {
        self.state_hash
    }

    /// Get the era id.
    pub fn era_id(&self) -> consensus::EraId {
        self.era_id
    }

    /// Get the protocol version.
    pub fn protocol_version(&self) -> ProtocolVersion {
        self.protocol_version
    }
}

impl From<ValidatorWeightsByEraIdRequest> for GetEraValidatorsRequest {
    fn from(input: ValidatorWeightsByEraIdRequest) -> Self {
        GetEraValidatorsRequest::new(input.state_hash, input.protocol_version)
    }
}

/// Request for era validators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EraValidatorsRequest {
    state_hash: Blake2bHash,
    protocol_version: ProtocolVersion,
}

impl EraValidatorsRequest {
    /// Constructs a new EraValidatorsRequest.
    pub fn new(state_hash: Blake2bHash, protocol_version: ProtocolVersion) -> Self {
        EraValidatorsRequest {
            state_hash,
            protocol_version,
        }
    }

    /// Get the state hash.
    pub fn state_hash(&self) -> Blake2bHash {
        self.state_hash
    }

    /// Get the protocol version.
    pub fn protocol_version(&self) -> ProtocolVersion {
        self.protocol_version
    }
}

impl From<EraValidatorsRequest> for GetEraValidatorsRequest {
    fn from(input: EraValidatorsRequest) -> Self {
        GetEraValidatorsRequest::new(input.state_hash, input.protocol_version)
    }
}

/// Request for auction info for a specific era.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuctionInfoByEraIdRequest {
    state_hash: Blake2bHash,
    era_id: auction::EraId,
    protocol_version: ProtocolVersion,
}

impl AuctionInfoByEraIdRequest {
    /// Constructs a new [`AuctionInfoByEraIdRequest`].
    pub fn new(
        state_hash: Blake2bHash,
        era_id: auction::EraId,
        protocol_version: ProtocolVersion,
    ) -> Self {
        AuctionInfoByEraIdRequest {
            state_hash,
            era_id,
            protocol_version,
        }
    }

    /// Get the state hash.
    pub fn state_hash(&self) -> Blake2bHash {
        self.state_hash
    }

    /// Get the era id.
    pub fn era_id(&self) -> auction::EraId {
        self.era_id
    }

    /// Get the protocol version.
    pub fn protocol_version(&self) -> ProtocolVersion {
        self.protocol_version
    }
}
