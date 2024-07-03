use crate::storage_types::{DataKey, TokenId};
use soroban_sdk::{Address, Env};

pub fn read_approved(env: &Env, token_id: TokenId) -> Option<Address> {
    let key = DataKey::Approved(token_id);
    env.storage().persistent().get::<DataKey, Address>(&key)
}

pub fn write_approved(env: &Env, token_id: TokenId, approved: Option<Address>) {
    let key = DataKey::Approved(token_id);
    match approved {
        Some(addr) => env.storage().persistent().set(&key, &addr),
        None => env.storage().persistent().remove(&key),
    }
}

pub fn read_approval_for_all(env: &Env, owner: Address, operator: Address) -> bool {
    let key = DataKey::ApprovalForAll(owner, operator);
    env.storage().persistent().get::<DataKey, bool>(&key).unwrap_or(false)
}

pub fn write_approval_for_all(env: &Env, owner: Address, operator: Address, approved: bool) {
    let key = DataKey::ApprovalForAll(owner, operator);
    env.storage().persistent().set(&key, &approved);
}