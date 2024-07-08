use soroban_sdk::{Address, Env};
use crate::storage_types::DataKey;

pub fn read_user_level(e: &Env, user: Address) -> u32 {
    e.storage()
        .persistent()
        .get(&DataKey::UserLevel(user))
        .unwrap_or(0)
}