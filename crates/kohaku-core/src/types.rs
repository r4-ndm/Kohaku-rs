use alloy_primitives::{Address as AlloyAddress, ChainId as AlloyChainId};
use serde::{Deserialize, Serialize};

/// CAIP-style asset identifier, e.g. `erc20:0x…` or native `slip44:60`.
pub type AssetId = String;

pub type ChainId = AlloyChainId;
pub type Address = AlloyAddress;
pub type Hex = String;

/// Amount of a given asset (on-chain smallest unit).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetAmount {
    pub asset: AssetId,
    pub amount: u128,
}

/// Opaque serialized public transaction (shield deposit, etc.).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreparedPublicOperation {
    pub payload: serde_json::Value,
}

/// Opaque serialized private operation (transfer / unshield).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreparedPrivateOperation {
    pub payload: serde_json::Value,
}
