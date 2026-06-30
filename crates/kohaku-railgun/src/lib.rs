//! Railgun integration for desktop Kohaku wallets.
//!
//! The Ethereum Foundation maintains a native Rust implementation at
//! [`ethereum/kohaku` → `crates/railgun`](https://github.com/ethereum/kohaku/tree/master/crates/railgun)
//! plus the npm package [`@kohaku-eth/railgun`](https://www.npmjs.com/package/@kohaku-eth/railgun)
//! (`crates/railgun-ts`). This crate wraps the official Rust crate (via git dependency) and
//! exposes [`kohaku_core::PrivacyPlugin`] for Vaughan and other Dioxus apps.

use std::sync::Arc;
use tokio::sync::Mutex;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;

use kohaku_core::{
    KohakuError, KohakuResult, PreparedPrivateOperation, PreparedPublicOperation,
    PrivacyPlugin, PrivacyPluginFactory, TxFeatures, Host, EthereumProvider, Storage,
};
use kohaku_core::{Address, AssetAmount};

use railgun::builder::RailgunBuilder;
use railgun::provider::RailgunProvider;
use railgun::account::signer::{PrivateKeySigner, RailgunSigner};
use railgun::chain_config::ChainConfig;
use railgun::crypto::keys::{HexKey, SpendingKey, ViewingKey};
use railgun::caip::AssetId as RailgunAssetId;
use railgun::database::Database as RailgunDatabase;
use railgun::database::DatabaseError as RailgunDatabaseError;

use eip_1193_provider::provider::{Eip1193Provider, Eip1193Error, RawLog};
use rand::SeedableRng;
use rand::rngs::StdRng;

/// Database adapter that routes calls to `kohaku_core::Storage`.
pub struct StorageDatabase {
    storage: Arc<dyn Storage>,
    prefix: String,
}

#[async_trait]
impl RailgunDatabase for StorageDatabase {
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, RailgunDatabaseError> {
        let hex_key = format!("{}:{}", self.prefix, hex::encode(key));
        match self.storage.get(&hex_key) {
            Ok(Some(val)) => {
                if val.is_empty() {
                    Ok(None)
                } else {
                    let bytes = hex::decode(&val).map_err(|e| RailgunDatabaseError::StorageError(e.to_string()))?;
                    Ok(Some(bytes))
                }
            }
            Ok(None) => Ok(None),
            Err(e) => Err(RailgunDatabaseError::StorageError(e.to_string())),
        }
    }

    async fn set(&self, key: &[u8], value: &[u8]) -> Result<(), RailgunDatabaseError> {
        let hex_key = format!("{}:{}", self.prefix, hex::encode(key));
        let hex_val = hex::encode(value);
        self.storage.set(&hex_key, &hex_val)
            .map_err(|e| RailgunDatabaseError::StorageError(e.to_string()))
    }

    async fn delete(&self, key: &[u8]) -> Result<(), RailgunDatabaseError> {
        let hex_key = format!("{}:{}", self.prefix, hex::encode(key));
        self.storage.set(&hex_key, "")
            .map_err(|e| RailgunDatabaseError::StorageError(e.to_string()))
    }
}

/// Provider adapter that routes calls from `eip_1193_provider` to `kohaku_core::EthereumProvider`.
pub struct ProviderAdapter {
    provider: Arc<dyn EthereumProvider>,
}

impl ProviderAdapter {
    pub fn new(provider: Arc<dyn EthereumProvider>) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl Eip1193Provider for ProviderAdapter {
    async fn get_chain_id(&self) -> Result<u64, Eip1193Error> {
        let res = self.provider.request("eth_chainId", json!([])).await
            .map_err(|e| Eip1193Error::Rpc(e.to_string()))?;
        let chain_id_hex = res.as_str().ok_or_else(|| Eip1193Error::Decode("Expected hex string".into()))?;
        u64::from_str_radix(chain_id_hex.trim_start_matches("0x"), 16)
            .map_err(|e| Eip1193Error::Decode(e.to_string()))
    }

    async fn get_block_number(&self) -> Result<u64, Eip1193Error> {
        let res = self.provider.request("eth_blockNumber", json!([])).await
            .map_err(|e| Eip1193Error::Rpc(e.to_string()))?;
        let block_hex = res.as_str().ok_or_else(|| Eip1193Error::Decode("Expected hex string".into()))?;
        u64::from_str_radix(block_hex.trim_start_matches("0x"), 16)
            .map_err(|e| Eip1193Error::Decode(e.to_string()))
    }

    async fn logs(
        &self,
        address: Address,
        event_signature: Option<alloy_primitives::FixedBytes<32>>,
        from_block: Option<u64>,
        to_block: Option<u64>,
    ) -> Result<Vec<RawLog>, Eip1193Error> {
        let mut filter = serde_json::Map::new();
        filter.insert("address".to_string(), json!(address));
        if let Some(sig) = event_signature {
            filter.insert("topics".to_string(), json!([sig]));
        }
        if let Some(from) = from_block {
            filter.insert("fromBlock".to_string(), json!(format!("0x{:x}", from)));
        }
        if let Some(to) = to_block {
            filter.insert("toBlock".to_string(), json!(format!("0x{:x}", to)));
        }

        let res = self.provider.request("eth_getLogs", json!([filter])).await
            .map_err(|e| Eip1193Error::Rpc(e.to_string()))?;

        let logs_val = res.as_array().ok_or_else(|| Eip1193Error::Decode("Expected logs array".into()))?;
        let mut raw_logs = Vec::with_capacity(logs_val.len());
        for val in logs_val {
            let address: Address = serde_json::from_value(val.get("address").cloned().unwrap_or(serde_json::Value::Null))
                .map_err(|e| Eip1193Error::Decode(e.to_string()))?;
            let topics: Vec<alloy_primitives::FixedBytes<32>> = serde_json::from_value(val.get("topics").cloned().unwrap_or(serde_json::Value::Null))
                .map_err(|e| Eip1193Error::Decode(e.to_string()))?;
            let data: alloy_primitives::Bytes = serde_json::from_value(val.get("data").cloned().unwrap_or(serde_json::Value::Null))
                .map_err(|e| Eip1193Error::Decode(e.to_string()))?;

            let block_number = if let Some(bn_val) = val.get("blockNumber") {
                if let Some(s) = bn_val.as_str() {
                    Some(u64::from_str_radix(s.trim_start_matches("0x"), 16)
                        .map_err(|e| Eip1193Error::Decode(e.to_string()))?)
                } else if let Some(n) = bn_val.as_u64() {
                    Some(n)
                } else {
                    None
                }
            } else {
                None
            };

            let block_timestamp = if let Some(bt_val) = val.get("blockTimestamp") {
                if let Some(s) = bt_val.as_str() {
                    Some(u64::from_str_radix(s.trim_start_matches("0x"), 16)
                        .map_err(|e| Eip1193Error::Decode(e.to_string()))?)
                } else if let Some(n) = bt_val.as_u64() {
                    Some(n)
                } else {
                    None
                }
            } else {
                None
            };

            let transaction_hash = if let Some(th_val) = val.get("transactionHash") {
                serde_json::from_value(th_val.clone()).ok()
            } else {
                None
            };

            raw_logs.push(RawLog {
                block_number,
                block_timestamp,
                transaction_hash,
                address,
                topics,
                data,
            });
        }
        Ok(raw_logs)
    }

    async fn eth_call(&self, to: Address, data: alloy_primitives::Bytes) -> Result<alloy_primitives::Bytes, Eip1193Error> {
        let req = json!({
            "to": to,
            "data": data,
        });
        let res = self.provider.request("eth_call", json!([req, "latest"])).await
            .map_err(|e| Eip1193Error::Rpc(e.to_string()))?;
        serde_json::from_value(res).map_err(|e| Eip1193Error::Decode(e.to_string()))
    }

    async fn estimate_gas(
        &self,
        to: Address,
        data: alloy_primitives::Bytes,
        from: Option<Address>,
    ) -> Result<u64, Eip1193Error> {
        let mut req = serde_json::Map::new();
        req.insert("to".to_string(), json!(to));
        req.insert("data".to_string(), json!(data));
        if let Some(f) = from {
            req.insert("from".to_string(), json!(f));
        }
        let res = self.provider.request("eth_estimateGas", json!([req])).await
            .map_err(|e| Eip1193Error::Rpc(e.to_string()))?;
        let gas_hex = res.as_str().ok_or_else(|| Eip1193Error::Decode("Expected hex string".into()))?;
        u64::from_str_radix(gas_hex.trim_start_matches("0x"), 16)
            .map_err(|e| Eip1193Error::Decode(e.to_string()))
    }

    async fn gas_price(&self) -> Result<u128, Eip1193Error> {
        let res = self.provider.request("eth_gasPrice", json!([])).await
            .map_err(|e| Eip1193Error::Rpc(e.to_string()))?;
        let price_hex = res.as_str().ok_or_else(|| Eip1193Error::Decode("Expected hex string".into()))?;
        u128::from_str_radix(price_hex.trim_start_matches("0x"), 16)
            .map_err(|e| Eip1193Error::Decode(e.to_string()))
    }

    async fn transaction_count(
        &self,
        address: Address,
        block: Option<u64>,
    ) -> Result<u64, Eip1193Error> {
        let block_str = match block {
            Some(b) => format!("0x{:x}", b),
            None => "latest".to_string(),
        };
        let res = self.provider.request("eth_getTransactionCount", json!([address, block_str])).await
            .map_err(|e| Eip1193Error::Rpc(e.to_string()))?;
        let count_hex = res.as_str().ok_or_else(|| Eip1193Error::Decode("Expected hex string".into()))?;
        u64::from_str_radix(count_hex.trim_start_matches("0x"), 16)
            .map_err(|e| Eip1193Error::Decode(e.to_string()))
    }
}

/// Thread-safe wrapper around `RailgunProvider`.
pub struct SendSyncProvider(pub RailgunProvider);

// SAFETY: `RailgunProvider` is functionally thread-safe on native platforms (containing
// read-only key material and thread-safe channels/state). However, because the upstream
// `RailgunSigner` trait object does not enforce `Send + Sync` bounds, the compiler cannot
// auto-derive these bounds for `RailgunProvider`. This is a temporary bridge until the
// upstream `MaybeSend` patch is merged in the official repository.
unsafe impl Send for SendSyncProvider {}
// SAFETY: Same as above. The inner structures utilize concurrent-safe types and are accessed
// read-only across async bounds.
unsafe impl Sync for SendSyncProvider {}

impl SendSyncProvider {
    pub async fn sync(&mut self) -> Result<(), railgun::provider::RailgunProviderError> {
        self.0.sync().await
    }

    pub async fn balance(&mut self, address: railgun::account::address::RailgunAddress) -> Vec<railgun::provider::BalanceEntry> {
        self.0.balance(address).await
    }

    pub fn shield(&self) -> railgun::transact::ShieldBuilder {
        self.0.shield()
    }

    pub fn transact(&self) -> railgun::transact::TransactionBuilder {
        self.0.transact()
    }

    pub async fn build<R: rand::Rng>(&mut self, builder: railgun::transact::TransactionBuilder, rng: &mut R) -> Result<railgun::transact::proved_transaction::ProvedTx, railgun::provider::RailgunProviderError> {
        self.0.build(builder, rng).await
    }

    pub async fn register(&mut self, signer: Arc<dyn RailgunSigner>) -> Result<(), railgun::provider::RailgunProviderError> {
        self.0.register(signer).await
    }
}

/// Serialized payload for reconstructable private operations.
#[derive(Serialize, Deserialize, Debug)]
pub struct PrivateOpPayload {
    pub kind: PrivateOpKind,
    pub intents: Vec<PrivateIntent>,
    pub recipient_address: Option<Address>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PrivateOpKind {
    Transfer,
    Unshield,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PrivateIntent {
    pub asset: String,
    pub amount: u128,
    pub to_railgun_address: String,
}

/// The Railgun plugin implementation.
pub struct RailgunPlugin {
    pub provider: Arc<Mutex<SendSyncProvider>>,
    pub signer: Arc<PrivateKeySigner>,
    pub host_provider: Arc<dyn EthereumProvider>,
}

#[async_trait]
impl PrivacyPlugin for RailgunPlugin {
    async fn instance_id(&self) -> KohakuResult<String> {
        Ok(self.signer.address().to_string())
    }

    fn features(&self) -> TxFeatures {
        TxFeatures {
            prepare_shield: true,
            prepare_shield_multi: true,
            prepare_transfer: true,
            prepare_transfer_multi: true,
            prepare_unshield: true,
            prepare_unshield_multi: true,
        }
    }

    async fn balance(&self, _assets: Option<Vec<kohaku_core::AssetId>>) -> KohakuResult<Vec<AssetAmount>> {
        let mut provider = self.provider.lock().await;
        provider.sync().await.map_err(|e| KohakuError::Provider(e.to_string()))?;
        let balances = provider.balance(self.signer.address()).await;
        
        let mut out = Vec::new();
        for b in balances {
            out.push(AssetAmount {
                asset: b.asset.to_string(),
                amount: b.amount,
            });
        }
        Ok(out)
    }

    async fn prepare_shield(
        &self,
        asset: AssetAmount,
        to: Option<String>,
    ) -> KohakuResult<PreparedPublicOperation> {
        self.prepare_shield_multi(vec![asset], to).await
    }

    async fn prepare_shield_multi(
        &self,
        assets: Vec<AssetAmount>,
        to: Option<String>,
    ) -> KohakuResult<PreparedPublicOperation> {
        let recipient = match to {
            Some(to_str) => to_str.parse::<railgun::account::address::RailgunAddress>()
                .map_err(|e| KohakuError::InvalidAddress(e.to_string()))?,
            None => self.signer.address(),
        };

        let provider = self.provider.lock().await;
        let mut builder = provider.shield();

        for asset in assets {
            if asset.asset.to_lowercase() == "slip44:60" || asset.asset.to_lowercase() == "native" {
                builder = builder.shield_native(recipient, asset.amount);
            } else {
                let parsed_asset = asset.asset.parse::<RailgunAssetId>()
                    .map_err(|_| KohakuError::UnsupportedAsset(asset.asset.clone()))?;
                builder = builder.shield(recipient, parsed_asset, asset.amount);
            }
        }

        let mut rng = StdRng::from_rng(&mut rand::rng());
        let txs = builder.build(&mut rng)
            .map_err(|e| KohakuError::Other(Box::new(e)))?;

        Ok(PreparedPublicOperation {
            payload: serde_json::to_value(txs)
                .map_err(|e| KohakuError::Other(Box::new(e)))?,
        })
    }

    async fn prepare_transfer(
        &self,
        asset: AssetAmount,
        to: String,
    ) -> KohakuResult<PreparedPrivateOperation> {
        self.prepare_transfer_multi(vec![asset], to).await
    }

    async fn prepare_transfer_multi(
        &self,
        assets: Vec<AssetAmount>,
        to: String,
    ) -> KohakuResult<PreparedPrivateOperation> {
        let mut intents = Vec::new();
        for asset in assets {
            intents.push(PrivateIntent {
                asset: asset.asset.clone(),
                amount: asset.amount,
                to_railgun_address: to.clone(),
            });
        }
        let payload = PrivateOpPayload {
            kind: PrivateOpKind::Transfer,
            intents,
            recipient_address: None,
        };
        Ok(PreparedPrivateOperation {
            payload: serde_json::to_value(payload)
                .map_err(|e| KohakuError::Other(Box::new(e)))?,
        })
    }

    async fn prepare_unshield(
        &self,
        asset: AssetAmount,
        to: Address,
    ) -> KohakuResult<PreparedPrivateOperation> {
        self.prepare_unshield_multi(vec![asset], to).await
    }

    async fn prepare_unshield_multi(
        &self,
        assets: Vec<AssetAmount>,
        to: Address,
    ) -> KohakuResult<PreparedPrivateOperation> {
        let mut intents = Vec::new();
        for asset in assets {
            intents.push(PrivateIntent {
                asset: asset.asset.clone(),
                amount: asset.amount,
                to_railgun_address: "".to_string(),
            });
        }
        let payload = PrivateOpPayload {
            kind: PrivateOpKind::Unshield,
            intents,
            recipient_address: Some(to),
        };
        Ok(PreparedPrivateOperation {
            payload: serde_json::to_value(payload)
                .map_err(|e| KohakuError::Other(Box::new(e)))?,
        })
    }

    async fn broadcast_private_operation(
        &self,
        operation: PreparedPrivateOperation,
    ) -> KohakuResult<()> {
        let payload: PrivateOpPayload = serde_json::from_value(operation.payload)
            .map_err(|e| KohakuError::Other(Box::new(e)))?;

        let mut provider = self.provider.lock().await;
        // Sync before building/proving
        provider.sync().await.map_err(|e| KohakuError::Provider(e.to_string()))?;
        
        let mut builder = provider.transact();
        match payload.kind {
            PrivateOpKind::Transfer => {
                for intent in payload.intents {
                    let parsed_asset = intent.asset.parse::<RailgunAssetId>()
                        .map_err(|_| KohakuError::UnsupportedAsset(intent.asset))?;
                    let to_rg = intent.to_railgun_address.parse::<railgun::account::address::RailgunAddress>()
                        .map_err(|e| KohakuError::InvalidAddress(e.to_string()))?;
                    builder = builder.transfer(self.signer.clone(), to_rg, parsed_asset, intent.amount, "");
                }
            }
            PrivateOpKind::Unshield => {
                let to_addr = payload.recipient_address.ok_or_else(|| KohakuError::Other("Missing recipient address for unshield".into()))?;
                for intent in payload.intents {
                    let parsed_asset = intent.asset.parse::<RailgunAssetId>()
                        .map_err(|_| KohakuError::UnsupportedAsset(intent.asset))?;
                    builder = builder.unshield(self.signer.clone(), to_addr, parsed_asset, intent.amount)
                        .map_err(|e| KohakuError::Other(Box::new(e)))?;
                }
            }
        }

        let mut rng = StdRng::from_rng(&mut rand::rng());
        let proved_tx = provider.build(builder, &mut rng).await
            .map_err(|e| KohakuError::Other(Box::new(e)))?;

        let tx_data = proved_tx.tx_data;
        let tx_param = json!({
            "to": tx_data.to,
            "data": format!("0x{}", hex::encode(&tx_data.data)),
            "value": format!("0x{:x}", tx_data.value),
        });

        self.host_provider.request("eth_sendTransaction", json!([tx_param])).await
            .map_err(|e| KohakuError::Provider(e.to_string()))?;

        Ok(())
    }
}

/// Configuration parameters for constructing the Railgun plugin.
#[derive(Clone, Debug, Default)]
pub struct RailgunPluginConfig {
    pub key_index: Option<u32>,
    pub poi: Option<bool>,
}

pub struct RailgunPluginFactory;

#[async_trait]
impl PrivacyPluginFactory for RailgunPluginFactory {
    type Plugin = RailgunPlugin;
    type Params = RailgunPluginConfig;

    async fn create<N, S, K, P>(
        &self,
        host: Host<N, S, K, P>,
        params: Self::Params,
    ) -> KohakuResult<Self::Plugin>
    where
        N: kohaku_core::host::Network + 'static,
        S: kohaku_core::host::Storage + 'static,
        K: kohaku_core::host::Keystore + 'static,
        P: kohaku_core::host::EthereumProvider + 'static,
    {
        let key_index = params.key_index.unwrap_or(0);
        let spending_key_path = railgun::account::signer::spending_key_path(key_index);
        let viewing_key_path = railgun::account::signer::viewing_key_path(key_index);
        
        let spending_key_hex = host.keystore.derive_at(&spending_key_path)
            .map_err(|e| KohakuError::Keystore(e.to_string()))?;
        let viewing_key_hex = host.keystore.derive_at(&viewing_key_path)
            .map_err(|e| KohakuError::Keystore(e.to_string()))?;

        let spending_key = SpendingKey::from_hex(&spending_key_hex)
            .map_err(|e| KohakuError::Other(Box::new(e)))?;
        let viewing_key = ViewingKey::from_hex(&viewing_key_hex)
            .map_err(|e| KohakuError::Other(Box::new(e)))?;

        let arc_provider = Arc::new(host.provider);
        let provider_adapter = ProviderAdapter::new(arc_provider.clone());
        let chain_id_val = provider_adapter.get_chain_id().await
            .map_err(|e| KohakuError::Provider(e.to_string()))?;

        let chain_config = ChainConfig::from_chain_id(alloy_primitives::ChainId::from(chain_id_val))
            .ok_or_else(|| KohakuError::UnsupportedChain(chain_id_val))?;

        let storage_db = StorageDatabase {
            storage: Arc::new(host.storage),
            prefix: format!("railgun:{}", chain_id_val),
        };

        let mut builder = RailgunBuilder::new(chain_config, Arc::new(provider_adapter));
        builder = builder.with_database(Arc::new(storage_db));

        if params.poi.unwrap_or(true) {
            builder = builder.with_poi();
        }

        let mut provider = builder.build().await
            .map_err(|e| KohakuError::Other(Box::new(e)))?;

        let signer = PrivateKeySigner::new_evm(spending_key, viewing_key, chain_id_val);
        provider.register(signer.clone()).await
            .map_err(|e| KohakuError::Other(Box::new(e)))?;

        Ok(RailgunPlugin {
            provider: Arc::new(Mutex::new(SendSyncProvider(provider))),
            signer,
            host_provider: arc_provider,
        })
    }
}
