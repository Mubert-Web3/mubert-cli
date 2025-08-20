pub mod api;
pub mod create_authority;
pub mod update_ip;

#[subxt::subxt(
    runtime_metadata_path = "ip_onchain_metadata.scale",
    derive_for_all_types = "Clone",
    derive_for_type(
        path = "bounded_collections::bounded_vec::BoundedVec",
        derive = "serde::Deserialize, serde::Serialize"
    ),
    derive_for_type(
        path = "pallet_ip_onchain::types::IPEntityKind",
        derive = "serde::Deserialize, serde::Serialize"
    ),
    derive_for_type(
        path = "pallet_ip_onchain::types::MetadataStandard",
        derive = "serde::Deserialize, serde::Serialize"
    ),
    derive_for_type(
        path = "pallet_ip_onchain::types::Wallet",
        derive = "serde::Deserialize, serde::Serialize"
    ),
    derive_for_type(
        path = "sp_core::crypto::AccountId32",
        derive = "serde::Deserialize, serde::Serialize"
    ),
    derive_for_type(path = "pallet_arweave::types::TaskState", derive = "PartialEq"),
    derive_for_type(
        path = "pallet_ip_onchain::types::AuthorityKind",
        derive = "clap::ValueEnum"
    )
)]
pub mod ip_onchain_runtime {}

use crate::ip_onchain_runtime::ip_onchain::calls::types::create_entity::Url;
use crate::ip_onchain_runtime::runtime_types::pallet_ip_onchain::types::MetadataFeature;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize)]
pub struct SecretKeyFile {
    #[serde(rename = "secretPhrase")]
    pub secret_phrase: String,
}

pub trait Bitmask {
    fn bitmask(&self) -> u64;
}

pub fn calculate_flags<T>(flags: Vec<String>) -> u64
where
    T: FromStr,
    T: Bitmask,
{
    let mut result = 0u64;
    for flag_str in flags {
        if let Ok(flag) = T::from_str(&flag_str) {
            result |= flag.bitmask();
        }
    }
    result
}

impl From<String> for Url {
    fn from(item: String) -> Self {
        Url { 0: item.into() }
    }
}

impl Bitmask for MetadataFeature {
    fn bitmask(&self) -> u64 {
        match self {
            MetadataFeature::Immutable => 0x00000001,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseMetadataFeatureError;

impl FromStr for MetadataFeature {
    type Err = ParseMetadataFeatureError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Immutable" => Ok(MetadataFeature::Immutable),
            _ => Err(ParseMetadataFeatureError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_flag() {
        let flags = vec!["Immutable".to_string()];
        let result = calculate_flags::<MetadataFeature>(flags);
        assert_eq!(result, 0x00000001);
    }

    #[test]
    fn test_unknown_flag_ignored() {
        let flags = vec!["Immutable".to_string(), "ignore".to_string()];
        let result = calculate_flags::<MetadataFeature>(flags);
        assert_eq!(result, 0x00000001);
    }

    #[test]
    fn test_no_flags() {
        let flags: Vec<String> = vec![];
        let result = calculate_flags::<MetadataFeature>(flags);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_duplicate_flags() {
        let flags = vec!["Immutable".to_string(), "Immutable".to_string()];
        let result = calculate_flags::<MetadataFeature>(flags);
        assert_eq!(result, 0x00000001);
    }
}
