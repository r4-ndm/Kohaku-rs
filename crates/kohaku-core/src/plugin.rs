use async_trait::async_trait;

use crate::error::KohakuResult;
use crate::host::Host;
use crate::types::{
    Address, AssetAmount, PreparedPrivateOperation, PreparedPublicOperation,
};

/// Which transaction kinds a plugin supports (mirrors Kohaku plugin `features`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TxFeatures {
    pub prepare_shield: bool,
    pub prepare_shield_multi: bool,
    pub prepare_transfer: bool,
    pub prepare_transfer_multi: bool,
    pub prepare_unshield: bool,
    pub prepare_unshield_multi: bool,
}

/// Privacy protocol adapter (Railgun, Privacy Pools, …).
///
/// Account identifiers are protocol-specific (`0zk…`, pool ids, etc.).
#[async_trait]
pub trait PrivacyPlugin: Send + Sync {
    /// Protocol-specific instance id (e.g. Railgun 0zk address).
    async fn instance_id(&self) -> KohakuResult<String>;

    /// Supported transaction surface for this build.
    fn features(&self) -> TxFeatures;

    /// Read shielded / private balances for the given assets (`None` = all known).
    async fn balance(&self, assets: Option<Vec<crate::types::AssetId>>)
        -> KohakuResult<Vec<AssetAmount>>;

    async fn prepare_shield(
        &self,
        asset: AssetAmount,
        to: Option<String>,
    ) -> KohakuResult<PreparedPublicOperation>;

    async fn prepare_shield_multi(
        &self,
        assets: Vec<AssetAmount>,
        to: Option<String>,
    ) -> KohakuResult<PreparedPublicOperation>;

    async fn prepare_transfer(
        &self,
        asset: AssetAmount,
        to: String,
    ) -> KohakuResult<PreparedPrivateOperation>;

    async fn prepare_transfer_multi(
        &self,
        assets: Vec<AssetAmount>,
        to: String,
    ) -> KohakuResult<PreparedPrivateOperation>;

    async fn prepare_unshield(
        &self,
        asset: AssetAmount,
        to: Address,
    ) -> KohakuResult<PreparedPrivateOperation>;

    async fn prepare_unshield_multi(
        &self,
        assets: Vec<AssetAmount>,
        to: Address,
    ) -> KohakuResult<PreparedPrivateOperation>;

    /// Submit a prepared private operation (4337 userop, broadcaster, etc.).
    async fn broadcast_private_operation(
        &self,
        operation: PreparedPrivateOperation,
    ) -> KohakuResult<()>;
}

/// Constructs a plugin given host services and opaque protocol parameters.
#[async_trait]
pub trait PrivacyPluginFactory: Send + Sync {
    type Plugin: PrivacyPlugin;
    type Params: Send + Sync + 'static;

    async fn create<N, S, K, P>(
        &self,
        host: Host<N, S, K, P>,
        params: Self::Params,
    ) -> KohakuResult<Self::Plugin>
    where
        N: crate::host::Network + 'static,
        S: crate::host::Storage + 'static,
        K: crate::host::Keystore + 'static,
        P: crate::host::EthereumProvider + 'static;
}
