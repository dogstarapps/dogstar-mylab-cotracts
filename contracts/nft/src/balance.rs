use crate::storage_types::{DataKey, TokenId, BALANCE_BUMP_AMOUNT, BALANCE_LIFETIME_THRESHOLD};
use soroban_sdk::{Address, Env};

pub fn read_balance(env: &Env, addr: Address) -> u64 {
    let key = DataKey::Balance(addr);
    if let Some(balance) = env.storage().persistent().get::<DataKey, u64>(&key) {
        env.storage()
            .persistent()
            .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
        balance
    } else {
        0
    }
}

pub fn write_balance(env: &Env, addr: Address, amount: u64) {
    let key = DataKey::Balance(addr);
    env.storage().persistent().set(&key, &amount);
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

pub fn read_owner(env: &Env, token_id: TokenId) -> Address {
    let key = DataKey::Owner(token_id);
    env.storage().persistent().get::<DataKey, Address>(&key).unwrap()
}

pub fn write_owner(env: &Env, token_id: TokenId, owner: Option<Address>) {
    let key = DataKey::Owner(token_id);
    match owner {
        Some(addr) => {
            env.storage().persistent().set(&key, &addr);
            env.storage()
                .persistent()
                .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
        }
        None => {
            env.storage().persistent().remove(&key);
        }
    }
}