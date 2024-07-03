use soroban_sdk::{Env, String, contracttype};

use crate::storage_types::DataKey;

#[derive(Clone)]
#[contracttype]
pub struct NFTMetadata {
    pub name: String,
    pub symbol: String,
    pub base_uri: String,
}

pub fn write_metadata(e: &Env, metadata: NFTMetadata) {
    let key = DataKey::Metadata;
    e.storage().instance().set(&key, &metadata);
}

pub fn read_metadata(e: &Env) -> NFTMetadata {
    let key = DataKey::Metadata;
    e.storage().instance().get(&key).unwrap()
}