//! Railgun integration for desktop Kohaku wallets.
//!
//! The Ethereum Foundation maintains a native Rust implementation at
//! [`ethereum/kohaku` → `crates/railgun`](https://github.com/ethereum/kohaku/tree/master/crates/railgun)
//! plus the npm package [`@kohaku-eth/railgun`](https://www.npmjs.com/package/@kohaku-eth/railgun)
//! (`crates/railgun-ts`). This crate will wrap the official Rust crate (via git dependency) and
//! expose [`kohaku_core::PrivacyPlugin`] for Vaughan and other Dioxus apps.

use async_trait::async_trait;
use kohaku_core::{
    KohakuError, KohakuResult, PreparedPrivateOperation, PreparedPublicOperation, PrivacyPlugin,
    TxFeatures,
};
use kohaku_core::{Address, AssetAmount};

/// Placeholder Railgun plugin until `railgun` from `ethereum/kohaku` is wired in.
pub struct RailgunPlugin;

#[async_trait]
impl PrivacyPlugin for RailgunPlugin {
    async fn instance_id(&self) -> KohakuResult<String> {
        Err(KohakuError::NotImplemented("railgun::instance_id"))
    }

    fn features(&self) -> TxFeatures {
        TxFeatures {
            prepare_shield: true,
            prepare_unshield: true,
            ..Default::default()
        }
    }

    async fn balance(
        &self,
        _assets: Option<Vec<kohaku_core::AssetId>>,
    ) -> KohakuResult<Vec<AssetAmount>> {
        Err(KohakuError::NotImplemented("railgun::balance"))
    }

    async fn prepare_shield(
        &self,
        _asset: AssetAmount,
        _to: Option<String>,
    ) -> KohakuResult<PreparedPublicOperation> {
        Err(KohakuError::NotImplemented("railgun::prepare_shield"))
    }

    async fn prepare_shield_multi(
        &self,
        _assets: Vec<AssetAmount>,
        _to: Option<String>,
    ) -> KohakuResult<PreparedPublicOperation> {
        Err(KohakuError::NotImplemented("railgun::prepare_shield_multi"))
    }

    async fn prepare_transfer(
        &self,
        _asset: AssetAmount,
        _to: String,
    ) -> KohakuResult<PreparedPrivateOperation> {
        Err(KohakuError::NotImplemented("railgun::prepare_transfer"))
    }

    async fn prepare_transfer_multi(
        &self,
        _assets: Vec<AssetAmount>,
        _to: String,
    ) -> KohakuResult<PreparedPrivateOperation> {
        Err(KohakuError::NotImplemented("railgun::prepare_transfer_multi"))
    }

    async fn prepare_unshield(
        &self,
        _asset: AssetAmount,
        _to: Address,
    ) -> KohakuResult<PreparedPrivateOperation> {
        Err(KohakuError::NotImplemented("railgun::prepare_unshield"))
    }

    async fn prepare_unshield_multi(
        &self,
        _assets: Vec<AssetAmount>,
        _to: Address,
    ) -> KohakuResult<PreparedPrivateOperation> {
        Err(KohakuError::NotImplemented("railgun::prepare_unshield_multi"))
    }

    async fn broadcast_private_operation(
        &self,
        _operation: PreparedPrivateOperation,
    ) -> KohakuResult<()> {
        Err(KohakuError::NotImplemented("railgun::broadcast_private_operation"))
    }
}
