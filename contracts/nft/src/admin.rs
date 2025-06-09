use crate::storage_types::*;
use soroban_sdk::{symbol_short, Address, Env};

pub fn has_administrator(e: &Env) -> bool {
    let key = DataKey::Admin;
    e.storage().instance().has(&key)
}

pub fn read_administrator(e: &Env) -> Address {
    let key = DataKey::Admin;
    e.storage().instance().get(&key).unwrap()
}

pub fn write_administrator(env: &Env, id: &Address) {
    let key = DataKey::Admin;
    env.storage().instance().set(&key, id);
}

pub fn is_whitelisted(e: &Env, member: &Address) -> bool {
    e.storage()
        .persistent()
        .get(&DataKey::Whitelist(member.clone()))
        .unwrap_or(false)
}

pub fn write_config(e: &Env, config: &Config) {
    let key: DataKey = DataKey::Config;
    e.storage().persistent().set(&key, config);
}

pub fn read_config(e: &Env) -> Config {
    let key = DataKey::Config;
    e.storage().persistent().get(&key).unwrap()
}

pub fn write_balance(e: &Env, balance: &Balance) {
    // Balance(u32) to track the different tokens balance???
    let key = DataKey::Balance;
    e.storage().persistent().set(&key, balance);
}

pub fn read_balance(e: &Env) -> Balance {
    let key = DataKey::Balance;
    e.storage().persistent().get(&key).unwrap_or(Balance {
        admin_terry: 0,
        admin_power: 0,
        haw_ai_terry: 0,
        haw_ai_power: 0,
        haw_ai_xtar: 0,
        total_deck_power: 0,
    })
}

pub fn write_state(e: &Env, state: &State) {
    let key = DataKey::State;
    e.storage().persistent().set(&key, state);

    e.events().publish((symbol_short!("state"),), state.clone());
}

pub fn read_state(e: &Env) -> State {
    let key = DataKey::State;
    e.storage().persistent().get(&key).unwrap_or(State {
        total_offer: 0,
        total_demand: 0,
        total_interest: 0,
        total_loan_duration: 0,
        total_loan_count: 0,
        total_staked_power: 0,
        total_borrowed_power: 0,
    })
}

pub fn add_level(e: &Env, level: Level) -> u32 {
    let level_id = get_and_increase_level_id(&e);
    e.storage()
        .persistent()
        .set(&DataKey::Level(level_id), &level);

    level_id
}

pub fn update_level(e: &Env, level_id: u32, level: Level) {
    e.storage()
        .persistent()
        .set(&DataKey::Level(level_id), &level);
}

pub fn get_and_increase_level_id(env: &Env) -> u32 {
    let prev = env
        .storage()
        .persistent()
        .get(&DataKey::LevelId)
        .unwrap_or(0u32);

    env.storage()
        .persistent()
        .set(&DataKey::LevelId, &(prev + 1));
    prev + 1
}
