use soroban_sdk::{contracttype, Address, Env};

use crate::storage_types::{DataKey, TokenId, BALANCE_BUMP_AMOUNT, BALANCE_LIFETIME_THRESHOLD};

#[derive(Clone, PartialEq)]
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

#[derive(Clone, PartialEq)]
#[contracttype]
pub enum Action {
    None,
    Stake,
    Fight,
    Lend,
    Borrow,
    Burn,
    Deck,
}

#[derive(Clone)]
#[contracttype]
pub struct Card {
    pub dl_level: u32,
    pub power: u32,
    pub locked_by_action: Action,
}

#[derive(Clone)]
#[contracttype]
pub struct CardInfo {
    pub initial_power: u32,
    pub max_power: u32,
    pub price_xtar: i128,
    pub price_terry: i128,
}

impl CardInfo {
    pub fn get_default_card(category: Category) -> Self {
        Self {
            initial_power: 1000,
            max_power: 10000,
            price_terry: 100,
            price_xtar: 100,
        }
    }
}

pub fn write_nft(env: &Env, owner: Address, category: Category, token_id: TokenId, nft: Card) {
    let key = DataKey::Card(owner, category, token_id);
    env.storage().persistent().set(&key, &nft);
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

pub fn read_nft(env: &Env, owner: Address, category: Category, token_id: TokenId) -> Card {
    let key = DataKey::Card(owner, category, token_id);
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
    env.storage().persistent().get(&key).unwrap()
}

pub fn exists(env: &Env, owner: Address, category: Category, token_id: TokenId) -> bool {
    let key: DataKey = DataKey::Card(owner, category, token_id);
    env.storage().persistent().has(&key)    
}

pub fn remove_nft(env: &Env, owner: Address, category: Category, token_id: TokenId) {
    let key = DataKey::Card(owner, category, token_id);
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
    env.storage().persistent().remove(&key);
}