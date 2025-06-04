use soroban_sdk::{contracttype, Env, String};

use crate::storage_types::DataKey;

#[derive(Clone)]
#[contracttype]
pub struct SFTMetadata {
    pub name: String,
    pub symbol: String,
    pub base_uri: String,
}

pub fn write_metadata(e: &Env, metadata: SFTMetadata) {
    let key = DataKey::Metadata;
    e.storage().instance().set(&key, &metadata);
}

pub fn read_metadata(e: &Env) -> SFTMetadata {
    let key = DataKey::Metadata;
    e.storage().instance().get(&key).unwrap()
}
