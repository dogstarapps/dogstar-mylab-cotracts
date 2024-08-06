use crate::*;
use admin::{read_balance, write_balance};
use nft_info::{read_nft, write_nft, Action, Category};
use user_info::read_user;
use soroban_sdk::{contracttype, vec, Address, Env, Vec};
use storage_types::{DataKey, TokenId, BALANCE_BUMP_AMOUNT, BALANCE_LIFETIME_THRESHOLD};

#[contracttype]
#[derive(Clone, Eq, PartialEq)]
pub struct Deck {
    pub owner: Address,
    pub categories: Vec<Category>,
    pub token_ids: Vec<TokenId>,
    pub total_power: u32,
    pub haw_ai_percentage: u32,
    pub bonus: u32,
}

pub fn write_deck(env: Env, fee_payer: Address, deck: Deck) {
    let owner = read_user(&env, fee_payer).owner;

    let key = DataKey::Deck(owner.clone());
    env.storage().persistent().set(&key, &deck);
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);

    let key = DataKey::Decks;
    let mut decks = read_decks(env.clone());
    if let Some(pos) = decks.iter().position(|deck| deck.owner == owner) {
        decks.set(pos.try_into().unwrap(), deck)
    } else {
        decks.push_back(deck);
    }

    env.storage().persistent().set(&key, &decks);

    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

pub fn read_decks(env: Env) -> Vec<Deck> {
    let key = DataKey::Decks;
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or(vec![&env.clone()])
}

pub fn remove_deck(env: Env, fee_payer: Address) {
    let owner = read_user(&env, fee_payer).owner;
    let key = DataKey::Deck(owner.clone());
    env.storage().persistent().remove(&key);
    if env.storage().persistent().has(&key) {
        env.storage().persistent().extend_ttl(
            &key,
            BALANCE_LIFETIME_THRESHOLD,
            BALANCE_BUMP_AMOUNT,
        );
    }

    let key = DataKey::Decks;
    let mut decks = read_decks(env.clone());
    if let Some(pos) = decks.iter().position(|deck| deck.owner == owner) {
        decks.remove(pos.try_into().unwrap());
    }

    env.storage().persistent().set(&key, &decks);

    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

pub fn read_deck(env: Env, fee_payer: Address) -> Deck {
    let owner = read_user(&env, fee_payer).owner;

    let key = DataKey::Deck(owner);
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
    env.storage().persistent().get(&key).unwrap()
}

pub fn place(env: Env, fee_payer: Address, categories: Vec<Category>, token_ids: Vec<TokenId>) {
    
    fee_payer.require_auth();
    let owner = read_user(&env, fee_payer).owner;

    assert!(categories.len() == 4, "Must place exactly 4 cards");
    assert!(token_ids.len() == 4, "Must place exactly 4 cards");

    let mut unique_categories = vec![&env.clone()];
    let mut total_power = 0;
    for i in 0..4 {
        let category = categories.get(i).unwrap();
        let token_id = token_ids.get(i).unwrap();
        let mut nft = read_nft(&env, owner.clone(), category.clone(), token_id.clone());

        assert!(
            nft.locked_by_action == Action::None,
            "Locked by other action"
        );

        nft.locked_by_action = Action::Deck;
        write_nft(
            &env,
            owner.clone(),
            category.clone(),
            token_id.clone(),
            nft.clone(),
        );

        if !unique_categories.contains(&category) {
            unique_categories.push_back(category.clone());
        }
        total_power += nft.power;
    }

    let bonus = match unique_categories.len() {
        1 => 0,
        2 => 5,
        3 => 10,
        4 => 25,
        _ => 0, // This should never happen
    };

    let mut balance = read_balance(&env);
    balance.total_deck_power += total_power;
    write_balance(&env, &balance);

    write_deck(
        env.clone(),
        owner.clone(),
        Deck {
            owner,
            haw_ai_percentage: 0,
            categories,
            token_ids,
            total_power,
            bonus,
        },
    );
    update_haw_ai_percentages(env);
}

pub fn update_place(env: Env, fee_payer: Address, categories: Vec<Category>, token_ids: Vec<TokenId>) {
    fee_payer.require_auth();
    let owner = read_user(&env, fee_payer).owner;

    assert!(categories.len() == 4, "Must place exactly 4 cards");
    assert!(token_ids.len() == 4, "Must place exactly 4 cards");

    let mut unique_categories = vec![&env.clone()];
    let mut total_power = 0;
    for i in 0..4 {
        let category = categories.get(i).unwrap();
        let token_id = token_ids.get(i).unwrap();
        let nft = read_nft(&env, owner.clone(), category.clone(), token_id.clone());

        assert!(
            nft.locked_by_action == Action::None || nft.locked_by_action == Action::Deck,
            "Locked by other action"
        );

        if !unique_categories.contains(&category) {
            unique_categories.push_back(category.clone());
        }
        total_power += nft.power;
    }

    let bonus = match unique_categories.len() {
        1 => 0,
        2 => 5,
        3 => 10,
        4 => 25,
        _ => 0, // This should never happen
    };

    let deck = read_deck(env.clone(), owner.clone());

    let mut balance = read_balance(&env);
    balance.total_deck_power -= deck.total_power;
    balance.total_deck_power += total_power;
    write_balance(&env, &balance);

    write_deck(
        env.clone(),
        owner.clone(),
        Deck {
            owner,
            haw_ai_percentage: 0,
            categories,
            token_ids,
            total_power,
            bonus,
        },
    );

    update_haw_ai_percentages(env);
}

pub fn remove_place(env: Env, fee_payer: Address) {
    fee_payer.require_auth();
    let owner = read_user(&env, fee_payer).owner;

    let deck = read_deck(env.clone(), owner.clone());

    for i in 0..4 {
        let category = deck.categories.get(i).unwrap();
        let token_id = deck.token_ids.get(i).unwrap();
        let mut nft = read_nft(&env, owner.clone(), category.clone(), token_id.clone());
        nft.locked_by_action = Action::None;
        write_nft(
            &env.clone(),
            owner.clone(),
            category.clone(),
            token_id.clone(),
            nft,
        );
    }

    let mut balance = read_balance(&env);
    balance.total_deck_power -= deck.total_power;
    write_balance(&env, &balance);

    remove_deck(env.clone(), owner.clone());
    update_haw_ai_percentages(env);
}

pub fn update_haw_ai_percentages(env: Env) {
    let decks = read_decks(env.clone());
    let balance = read_balance(&env);

    for mut deck in decks {
        let haw_ai_percentage = deck.total_power * (100 + deck.bonus) / balance.total_deck_power;

        deck.haw_ai_percentage = haw_ai_percentage;

        write_deck(env.clone(), deck.owner.clone(), deck);
    }
}
