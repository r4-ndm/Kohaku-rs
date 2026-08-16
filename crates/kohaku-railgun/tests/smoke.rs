//! Offline smoke test for the `kohaku-railgun` plugin integration.
//!
//! Exercises the plugin lifecycle that does not require a live chain:
//! factory construction (key derivation, signer registration, provider build),
//! instance id, feature flags, and the pure serialization / shield-building
//! paths. Balance syncing and private-operation broadcast require a live RPC +
//! indexer and are intentionally out of scope here.

use std::collections::HashMap;
use std::sync::Mutex;

use alloy_primitives::Address;
use async_trait::async_trait;
use serde_json::json;

use kohaku_core::host::{FetchInit, FetchResponse};
use kohaku_core::{
    AssetAmount, EthereumProvider, Hex, Host, Keystore, KohakuError, KohakuResult, Network,
    PrivacyPlugin, PrivacyPluginFactory, Storage, TxFeatures,
};
use kohaku_railgun::{RailgunPlugin, RailgunPluginConfig, RailgunPluginFactory};

/// Known-valid 32-byte keys (values taken from the upstream `PrivateKeySigner` test).
const SPENDING_KEY_HEX: &str = "039b3b11110e49d7340cbe7171791972e3c0d94ef31b18d6ab93d7ace62d278a";
const VIEWING_KEY_HEX: &str = "d345b2cc2f414aa93413b9572fa2b26e0e869e9274b006415a8d62ab1fa2dcb1";

/// In-memory key-value storage.
#[derive(Default)]
struct MockStorage(Mutex<HashMap<String, String>>);

impl Storage for MockStorage {
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

/// Keystore that returns fixed, valid railgun keys.
struct MockKeystore;

impl Keystore for MockKeystore {
    fn derive_at(&self, path: &str) -> KohakuResult<Hex> {
        if path.starts_with("m/420") {
            Ok(VIEWING_KEY_HEX.to_string())
        } else {
            Ok(SPENDING_KEY_HEX.to_string())
        }
    }
}

/// Provider that only answers `eth_chainId` (mainnet), enough to construct the plugin offline.
struct MockProvider;

#[async_trait]
impl EthereumProvider for MockProvider {
    async fn request(
        &self,
        method: &str,
        _params: serde_json::Value,
    ) -> KohakuResult<serde_json::Value> {
        match method {
            "eth_chainId" => Ok(json!("0x1")),
            other => Err(KohakuError::Provider(format!("unexpected method: {other}"))),
        }
    }
}

/// Network surface is unused by the Railgun plugin; a stub that never fetches.
struct MockNetwork;

#[async_trait]
impl Network for MockNetwork {
    async fn fetch(&self, _url: &str, _init: Option<FetchInit>) -> KohakuResult<FetchResponse> {
        Err(KohakuError::NotImplemented("mock network"))
    }
}

fn host() -> Host<MockNetwork, MockStorage, MockKeystore, MockProvider> {
    Host::new(
        MockNetwork,
        MockStorage::default(),
        MockKeystore,
        MockProvider,
    )
}

async fn create_plugin() -> RailgunPlugin {
    RailgunPluginFactory
        .create(
            host(),
            RailgunPluginConfig {
                key_index: Some(0),
                poi: Some(false),
            },
        )
        .await
        .expect("plugin construction should succeed offline")
}

#[tokio::test]
async fn creates_plugin_and_reports_instance_id() {
    let plugin = create_plugin().await;

    let id = plugin
        .instance_id()
        .await
        .expect("instance_id should succeed");
    assert!(
        id.starts_with("0zk"),
        "instance id should be a railgun address, got {id}"
    );
    assert!(id.len() > 3, "instance id should not be empty, got {id}");
}

#[tokio::test]
async fn reports_all_tx_features() {
    let plugin = create_plugin().await;

    assert_eq!(
        plugin.features(),
        TxFeatures {
            prepare_shield: true,
            prepare_shield_multi: true,
            prepare_transfer: true,
            prepare_transfer_multi: true,
            prepare_unshield: true,
            prepare_unshield_multi: true,
        }
    );
}

#[tokio::test]
async fn prepare_transfer_serializes_intent_payload() {
    let plugin = create_plugin().await;

    let op = plugin
        .prepare_transfer(
            AssetAmount {
                asset: "erc20:0x0000000000000000000000000000000000000001".to_string(),
                amount: 123,
            },
            "0zkrecipient".to_string(),
        )
        .await
        .expect("prepare_transfer should succeed");

    assert_eq!(op.payload["kind"], json!("Transfer"));
    assert_eq!(op.payload["intents"].as_array().unwrap().len(), 1);
    assert_eq!(op.payload["intents"][0]["amount"], json!(123));
    assert_eq!(
        op.payload["intents"][0]["to_railgun_address"],
        json!("0zkrecipient")
    );
    assert!(op.payload["recipient_address"].is_null());
}

#[tokio::test]
async fn prepare_unshield_serializes_recipient() {
    let plugin = create_plugin().await;

    let recipient = Address::repeat_byte(0xab);
    let op = plugin
        .prepare_unshield(
            AssetAmount {
                asset: "erc20:0x0000000000000000000000000000000000000001".to_string(),
                amount: 7,
            },
            recipient,
        )
        .await
        .expect("prepare_unshield should succeed");

    assert_eq!(op.payload["kind"], json!("Unshield"));
    assert_eq!(op.payload["intents"].as_array().unwrap().len(), 1);
    assert_eq!(op.payload["recipient_address"], json!(recipient));
}

#[tokio::test]
async fn prepare_shield_builds_native_shield_offline() {
    let plugin = create_plugin().await;

    let op = plugin
        .prepare_shield(
            AssetAmount {
                asset: "slip44:60".to_string(),
                amount: 1_000_000,
            },
            None,
        )
        .await
        .expect("prepare_shield should succeed offline");

    let txs = op
        .payload
        .as_array()
        .expect("shield payload should be an array of transactions");
    assert_eq!(txs.len(), 1, "native shield should produce a single tx");
    assert!(txs[0]["to"].is_string(), "tx should have a `to` address");
    assert!(txs[0]["data"].is_string(), "tx should have `data` calldata");
}

#[tokio::test]
async fn prepare_shield_builds_erc20_shield_offline() {
    let plugin = create_plugin().await;

    let op = plugin
        .prepare_shield(
            AssetAmount {
                asset: "erc20:0x0000000000000000000000000000000000000001".to_string(),
                amount: 500,
            },
            None,
        )
        .await
        .expect("prepare_shield should succeed offline");

    let txs = op
        .payload
        .as_array()
        .expect("shield payload should be an array of transactions");
    assert_eq!(txs.len(), 1, "erc20 shield should produce a single tx");
    assert!(txs[0]["to"].is_string(), "tx should have a `to` address");
    assert!(txs[0]["data"].is_string(), "tx should have `data` calldata");
}
