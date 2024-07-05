use soroban_sdk::{Address, Env};

use crate::storage_types::{DataKey, TokenId};

pub fn write_nft_level(e: &Env, token_id: TokenId, level: u64) {
    e.storage()
        .persistent()
        .set(&DataKey::NFTLevel(token_id), &level);
}

pub fn read_nft_level(e: &Env, token_id: TokenId) -> u64 {
    e.storage()
        .persistent()
        .get(&DataKey::NFTLevel(token_id))
        .unwrap_or(0)
}

pub fn write_nft_lock(e: &Env, token_id: TokenId, locker: Option<Address>) {
    e.storage()
        .persistent()
        .set(&DataKey::NFTLock(token_id), &locker);
}

pub fn read_nft_lock(e: &Env, token_id: TokenId) -> Option<Address> {
    e.storage()
        .persistent()
        .get(&DataKey::NFTLock(token_id))
        .unwrap_or(None)
}