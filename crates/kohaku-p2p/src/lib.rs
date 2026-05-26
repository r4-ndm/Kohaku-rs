//! Peer-to-peer transaction broadcasting (roadmap Phase 2).
//!
//! Upstream surface: [`packages/plugins/src/broadcaster`](https://github.com/ethereum/kohaku/tree/master/packages/plugins/src/broadcaster).

use async_trait::async_trait;
use kohaku_core::KohakuResult;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum P2pError {
    #[error("broadcast channel unavailable")]
    Unavailable,

    #[error("broadcast rejected: {0}")]
    Rejected(String),
}

/// Submits signed transactions outside the public mempool.
#[async_trait]
pub trait PrivateBroadcaster: Send + Sync {
    async fn broadcast(&self, signed_tx: &[u8]) -> KohakuResult<()>;
}

/// No-op broadcaster for local development.
pub struct NullBroadcaster;

#[async_trait]
impl PrivateBroadcaster for NullBroadcaster {
    async fn broadcast(&self, _signed_tx: &[u8]) -> KohakuResult<()> {
        Err(kohaku_core::KohakuError::NotImplemented("p2p::broadcast"))
    }
}
