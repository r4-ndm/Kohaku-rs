use thiserror::Error;

use crate::types::{AssetId, ChainId};

/// Unified error surface for Kohaku-compatible Rust wallets.
#[derive(Debug, Error)]
pub enum KohakuError {
    #[error("unsupported asset: {0}")]
    UnsupportedAsset(AssetId),

    #[error("unsupported chain: {0}")]
    UnsupportedChain(ChainId),

    #[error("invalid address: {0}")]
    InvalidAddress(String),

    #[error(
        "insufficient balance for {asset}: required {required}, available {available}"
    )]
    InsufficientBalance {
        asset: AssetId,
        required: u128,
        available: u128,
    },

    #[error("multiple assets are not supported by this plugin")]
    MultiAssetsNotSupported,

    #[error("feature not implemented: {0}")]
    NotImplemented(&'static str),

    #[error("provider error: {0}")]
    Provider(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("keystore error: {0}")]
    Keystore(String),

    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

pub type KohakuResult<T> = Result<T, KohakuError>;
