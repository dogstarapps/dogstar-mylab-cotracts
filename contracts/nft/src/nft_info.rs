use soroban_sdk::{contracttype, log, Address, Env};

use crate::storage_types::{DataKey, TokenId, BALANCE_BUMP_AMOUNT, BALANCE_LIFETIME_THRESHOLD};

#[derive(Clone, PartialEq)]
#[contracttype]
pub enum Category {
    Leader,
    Resource,
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

#[contracttype]
#[derive(Clone)]
pub struct Card {
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

pub fn write_nft(env: &Env, owner: Address, token_id: TokenId, card: Card) {
    log!(&env, "write_nft >> Write nft for {}, token id {}", owner.clone(), token_id.clone());
    let key = DataKey::Card( owner, token_id);
    env.storage().persistent().set(&key, &card);
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

pub fn read_nft(env: &Env, owner: Address,  token_id: TokenId,) -> Option<Card> {
    log!(&env, "read_nft >> Read nft for {}, token id {}", owner.clone(), token_id.clone());
    let key = DataKey::Card(owner, token_id);
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
    env.storage().persistent().get(&key).unwrap()
}

pub fn exists(env: &Env, owner: Address, token_id: TokenId) -> bool {
    let key: DataKey = DataKey::Card( owner, token_id.clone());
    env.storage().persistent().has(&key)    
}

pub fn remove_nft(env: &Env, owner: Address,  token_id: TokenId) {
    let key = DataKey::Card(owner, token_id.clone());
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
    env.storage().persistent().remove(&key);
}
