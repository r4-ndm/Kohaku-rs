//! Core traits and types for **kohaku-rs**, a community-driven Rust SDK aligned with the
//! [Ethereum Foundation Kohaku](https://github.com/ethereum/kohaku) privacy toolkit.
//!
//! The design mirrors [`@kohaku-eth/plugins`](https://github.com/ethereum/kohaku/tree/master/packages/plugins):
//! wallets implement a **Host**, privacy protocols implement **Plugins**, and desktop apps (e.g.
//! [Vaughan](https://github.com/r4-ndm/vaughan)) compose both without a browser runtime.

pub mod error;
pub mod host;
pub mod plugin;
pub mod types;

pub use error::{KohakuError, KohakuResult};
pub use host::{EthereumProvider, Host, Keystore, Network, Storage};
pub use plugin::{PrivacyPlugin, PrivacyPluginFactory, TxFeatures};
pub use types::{
    Address, AssetAmount, AssetId, ChainId, Hex, PreparedPrivateOperation,
    PreparedPublicOperation,
};
