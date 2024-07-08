use soroban_sdk::{contracttype, Address, Env};

use crate::storage_types::DataKey;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub locked_block: u32,
    pub haw_ai_pot: Address,
}

pub fn has_administrator(e: &Env) -> bool {
    let key = DataKey::Admin;
    e.storage().instance().has(&key)
}

pub fn read_administrator(e: &Env) -> Address {
    let key = DataKey::Admin;
    e.storage().instance().get(&key).unwrap()
}

pub fn write_administrator(e: &Env, id: &Address) {
    let key = DataKey::Admin;
    e.storage().instance().set(&key, id);
}

pub fn is_whitelisted(e: &Env, member: &Address) -> bool {
    e.storage()
        .persistent()
        .get(&DataKey::Whitelist(member.clone()))
        .unwrap_or(false)
}

pub fn write_config(e: &Env, config: &Config) {
    let key = DataKey::Config;
    e.storage()
        .persistent()
        .set(&key, config);
}

pub fn read_config(e: &Env) -> Config {
    let key = DataKey::Config;
    e.storage().persistent().get(&key).unwrap()
}