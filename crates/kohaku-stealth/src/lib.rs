//! ERC-5564 stealth address helpers for Kohaku-compatible desktop wallets.
//!
//! Upstream Kohaku does not yet ship a dedicated stealth package; this crate tracks the
//! [Kohaku roadmap](https://notes.ethereum.org/@niard/KohakuRoadmap) and reuses battle-tested
//! Rust crypto until EF types stabilize.

use eth_stealth_addresses::generate_stealth_meta_address;
use kohaku_core::KohakuResult;
use thiserror::Error;

pub use eth_stealth_addresses;

#[derive(Debug, Error)]
pub enum StealthError {
    #[error("stealth crypto error: {0}")]
    Crypto(String),
}

/// Spending / viewing meta-address material for ERC-5564 (secp256k1, scheme 1).
#[derive(Clone, Debug)]
pub struct StealthMetaAddress {
    pub stealth_meta_address_hex: String,
    pub spending_key_hex: String,
    pub viewing_key_hex: String,
}

/// Generate a fresh stealth meta-address (recipient registration).
pub fn generate_meta_address() -> KohakuResult<StealthMetaAddress> {
    let (stealth_meta_address, spending_key, viewing_key) = generate_stealth_meta_address();
    Ok(StealthMetaAddress {
        stealth_meta_address_hex: bytes_to_hex(&stealth_meta_address),
        spending_key_hex: bytes_to_hex(&spending_key),
        viewing_key_hex: bytes_to_hex(&viewing_key),
    })
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_meta_address() {
        let meta = generate_meta_address().expect("meta address");
        assert!(!meta.stealth_meta_address_hex.is_empty());
        assert!(!meta.spending_key_hex.is_empty());
        assert!(!meta.viewing_key_hex.is_empty());
    }
}
