use soroban_sdk::{contracttype, Address, Env};

use crate::storage_types::{DataKey, TokenId, BALANCE_BUMP_AMOUNT, BALANCE_LIFETIME_THRESHOLD};

#[derive(Clone)]
#[contracttype]
pub enum Category {
    Leader,
    Human,
    Skill,
    Weapon,
}

#[derive(Clone, PartialEq)]
#[contracttype]
pub enum Currency {
    Terry,
    Xtar,
}

#[derive(Clone)]
#[contracttype]
pub enum Action {
    None,
    Stake,
    Fight,
    LendAndBorrow,
    Burn,
    Deck,
}

#[derive(Clone)]
#[contracttype]
pub struct CardInfo {
    pub dl_level: u32,
    pub initial_power: u32,
    pub max_power: u32,
    pub price_xtar: i128,
    pub price_terry: i128,
    pub locked_by_action: Action,
}

impl CardInfo {
    pub fn get_default_card(category: Category) -> Self {
        Self {
            dl_level: 0,
            initial_power: 0,
            max_power: 100,
            price_terry: 100,
            price_xtar: 100,
            locked_by_action: Action::None,
        }
    }
}

pub fn write_nft(env: &Env, owner: Address, category: Category, token_id: TokenId, nft: CardInfo) {
    let key = DataKey::Card(owner, category, token_id);
    env.storage().persistent().set(&key, &nft);
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

pub fn read_nft(env: &Env, owner: Address, category: Category, token_id: TokenId) -> CardInfo {
    let key = DataKey::Card(owner, category, token_id);
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
    env.storage().persistent().get(&key).unwrap()
}

pub fn exists(env: &Env, owner: Address, category: Category, token_id: TokenId) -> bool {
    let key = DataKey::Card(owner, category, token_id);
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
    env.storage().persistent().has(&key)
}

pub fn remove_nft(env: &Env, owner: Address, category: Category, token_id: TokenId) {
    let key = DataKey::Card(owner, category, token_id);
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
    env.storage().persistent().remove(&key);
}