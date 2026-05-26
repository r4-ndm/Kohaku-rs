use async_trait::async_trait;

use crate::error::KohakuResult;
use crate::types::Hex;

/// HTTP / fetch surface used by plugins (mirrors Kohaku `Network`).
#[async_trait]
pub trait Network: Send + Sync {
    async fn fetch(&self, url: &str, init: Option<FetchInit>) -> KohakuResult<FetchResponse>;
}

#[derive(Clone, Debug, Default)]
pub struct FetchInit {
    pub method: Option<String>,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
}

#[derive(Clone, Debug)]
pub struct FetchResponse {
    pub status: u16,
    pub body: Vec<u8>,
}

/// Key-value storage scoped to a plugin instance (mirrors Kohaku `Storage`).
pub trait Storage: Send + Sync {
    fn set(&self, key: &str, value: &str) -> KohakuResult<()>;
    fn get(&self, key: &str) -> KohakuResult<Option<String>>;
}

/// HD derivation from the wallet mnemonic (mirrors Kohaku `Keystore`).
pub trait Keystore: Send + Sync {
    fn derive_at(&self, path: &str) -> KohakuResult<Hex>;
}

/// JSON-RPC style chain access (mirrors Kohaku `EthereumProvider`).
#[async_trait]
pub trait EthereumProvider: Send + Sync {
    async fn request(&self, method: &str, params: serde_json::Value) -> KohakuResult<serde_json::Value>;
}

/// Environment injected when constructing a privacy plugin.
pub struct Host<N, S, K, P> {
    pub network: N,
    pub storage: S,
    pub keystore: K,
    pub provider: P,
}

impl<N, S, K, P> Host<N, S, K, P>
where
    N: Network,
    S: Storage,
    K: Keystore,
    P: EthereumProvider,
{
    pub fn new(network: N, storage: S, keystore: K, provider: P) -> Self {
        Self {
            network,
            storage,
            keystore,
            provider,
        }
    }
}
