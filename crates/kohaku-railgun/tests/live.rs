//! Live-chain integration tests for `kohaku-railgun`.
//!
//! These exercise the paths the offline [`smoke`] test intentionally skips:
//! real indexer sync, shielded balance reads, and private-operation broadcast.
//! They are `#[ignore]`d because they require a reachable RPC endpoint plus a
//! funded (and, for broadcast, shielded) account on a live chain.
//!
//! # Running
//!
//! ```text
//! export KOHaku_RPC_URL="https://ethereum-sepolia-rpc.publicnode.com"
//! export KOHaku_SPENDING_KEY="<64-hex-char spending key>"
//! export KOHaku_VIEWING_KEY="<64-hex-char viewing key>"
//! cargo test -p kohaku-railgun --test live -- --ignored --nocapture
//! ```
//!
//! * [`live_balance_reads_shielded_balances`] needs only the RPC + the Railgun
//!   indexer for that chain (no funds). It may return an empty list for a fresh
//!   account.
//! * [`live_broadcast_private_transfer`] additionally needs the account to
//!   already hold shielded UTXOs (and gas ETH). Set `KOHaku_TRANSFER_AMOUNT`
//!   (u128) to control the transfer amount; it defaults to `1`.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use serde_json::json;

use kohaku_core::host::{FetchInit, FetchResponse};
use kohaku_core::{
    AssetAmount, EthereumProvider, Hex, Host, Keystore, KohakuError, KohakuResult, Network,
    PrivacyPlugin, PrivacyPluginFactory, Storage,
};
use kohaku_railgun::{RailgunPluginConfig, RailgunPluginFactory};

/// In-memory key-value storage. State is not persisted between runs, so each
/// run performs a full indexer sync.
#[derive(Default)]
struct MemoryStorage(Mutex<HashMap<String, String>>);

impl Storage for MemoryStorage {
    fn set(&self, key: &str, value: &str) -> KohakuResult<()> {
        self.0
            .lock()
            .unwrap()
            .insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn get(&self, key: &str) -> KohakuResult<Option<String>> {
        Ok(self.0.lock().unwrap().get(key).cloned())
    }
}

/// Keystore backed by `KOHaku_SPENDING_KEY` / `KOHaku_VIEWING_KEY` env vars.
struct EnvKeystore {
    spending: String,
    viewing: String,
}

impl Keystore for EnvKeystore {
    fn derive_at(&self, path: &str) -> KohakuResult<Hex> {
        if path.starts_with("m/420") {
            Ok(self.viewing.clone())
        } else {
            Ok(self.spending.clone())
        }
    }
}

/// JSON-RPC provider backed by `reqwest` against `KOHaku_RPC_URL`.
struct HttpProvider {
    client: reqwest::Client,
    url: String,
}

#[async_trait]
impl EthereumProvider for HttpProvider {
    async fn request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> KohakuResult<serde_json::Value> {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });
        let res = self
            .client
            .post(&self.url)
            .json(&body)
            .send()
            .await
            .map_err(|e| KohakuError::Provider(e.to_string()))?;
        let val: serde_json::Value = res
            .json()
            .await
            .map_err(|e| KohakuError::Provider(e.to_string()))?;
        if let Some(err) = val.get("error") {
            return Err(KohakuError::Provider(err.to_string()));
        }
        Ok(val
            .get("result")
            .cloned()
            .unwrap_or(serde_json::Value::Null))
    }
}

/// Network surface is unused by the Railgun plugin (sync uses the chain RPC and
/// the subsquid indexer directly); a stub that never fetches.
struct StubNetwork;

#[async_trait]
impl Network for StubNetwork {
    async fn fetch(&self, _url: &str, _init: Option<FetchInit>) -> KohakuResult<FetchResponse> {
        Err(KohakuError::NotImplemented("stub network"))
    }
}

async fn live_plugin() -> KohakuResult<kohaku_railgun::RailgunPlugin> {
    let rpc_url = std::env::var("KOHaku_RPC_URL").map_err(|_| {
        KohakuError::Other("KOHaku_RPC_URL must be set to a live RPC endpoint".into())
    })?;
    let spending = std::env::var("KOHaku_SPENDING_KEY")
        .map_err(|_| KohakuError::Other("KOHaku_SPENDING_KEY must be set (64 hex chars)".into()))?;
    let viewing = std::env::var("KOHaku_VIEWING_KEY")
        .map_err(|_| KohakuError::Other("KOHaku_VIEWING_KEY must be set (64 hex chars)".into()))?;

    let host = Host::new(
        StubNetwork,
        MemoryStorage::default(),
        EnvKeystore { spending, viewing },
        HttpProvider {
            client: reqwest::Client::new(),
            url: rpc_url,
        },
    );

    RailgunPluginFactory
        .create(
            host,
            RailgunPluginConfig {
                key_index: Some(0),
                poi: Some(true),
            },
        )
        .await
}

#[tokio::test]
#[ignore = "requires a live RPC + Railgun indexer (no funds needed); set KOHaku_RPC_URL, KOHaku_SPENDING_KEY, KOHaku_VIEWING_KEY"]
async fn live_balance_reads_shielded_balances() -> KohakuResult<()> {
    let plugin = live_plugin().await?;
    let balances = plugin.balance(None).await?;
    eprintln!("shielded balances: {balances:?}");
    Ok(())
}

#[tokio::test]
#[ignore = "broadcasts a real private tx; requires shielded UTXOs + gas ETH; set KOHaku_TRANSFER_AMOUNT (default 1)"]
async fn live_broadcast_private_transfer() -> KohakuResult<()> {
    let plugin = live_plugin().await?;

    let balances = plugin.balance(None).await?;
    let Some(first) = balances.first() else {
        eprintln!("account has no shielded balance; shield funds before running this test");
        return Ok(());
    };

    let amount: u128 = std::env::var("KOHaku_TRANSFER_AMOUNT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    // Transfer back to self to exercise prove + broadcast without a second party.
    let to_self = plugin.instance_id().await?;
    let op = plugin
        .prepare_transfer(
            AssetAmount {
                asset: first.asset.clone(),
                amount,
            },
            to_self,
        )
        .await?;

    plugin.broadcast_private_operation(op).await
}
