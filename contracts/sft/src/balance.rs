use crate::storage_types::{DataKey, TokenId, BALANCE_BUMP_AMOUNT, BALANCE_LIFETIME_THRESHOLD};
use soroban_sdk::{Address, Env};

pub fn read_balance(env: &Env, addr: Address, id: TokenId) -> u64 {
    let key = DataKey::Balance(addr, id);
    if let Some(balance) = env.storage().persistent().get::<DataKey, u64>(&key) {
        env.storage().persistent().extend_ttl(
            &key,
            BALANCE_LIFETIME_THRESHOLD,
            BALANCE_BUMP_AMOUNT,
        );
        balance
    } else {
        0
    }
}

pub fn write_balance(env: &Env, addr: Address, id: TokenId, amount: u64) {
    let key = DataKey::Balance(addr, id);
    env.storage().persistent().set(&key, &amount);
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}
