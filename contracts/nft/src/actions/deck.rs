use crate::*;
use admin::{read_balance, write_balance};
use contract::NFT;
use nft_info::{read_nft, write_nft, Action, Category};
use soroban_sdk::{contracttype, vec, Address, Env, Vec};
use storage_types::{DataKey, TokenId, BALANCE_BUMP_AMOUNT, BALANCE_LIFETIME_THRESHOLD};

#[contracttype]
#[derive(Clone, Eq, PartialEq)]
pub struct Deck {
    pub categories: Vec<Category>,
    pub token_ids: Vec<TokenId>,
    pub total_power: u32,
    pub haw_ai_percentage: u32,
}

pub fn write_deck(env: Env, owner: Address, deck: Deck) {
    let key = DataKey::Deck(owner);
    env.storage().persistent().set(&key, &deck);
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

pub fn remove_deck(env: Env, owner: Address) {
    let key = DataKey::Deck(owner);
    env.storage().persistent().remove(&key);
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

pub fn read_deck(env: Env, owner: Address) -> Deck {
    let key = DataKey::Deck(owner);
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
    env.storage().persistent().get(&key).unwrap()
}

impl NFT {
    pub fn place(env: Env, owner: Address, categories: Vec<Category>, token_ids: Vec<TokenId>) {
        owner.require_auth();

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
            write_nft(&env, owner.clone(), category.clone(), token_id.clone(), nft.clone());

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

        let haw_ai_percentage = ((total_power as f32) * (100 as f32)
            / (balance.total_deck_power as f32)
            * (1.0 + bonus as f32 / 100.0)) as u32;

        write_deck(env.clone(), owner.clone(), Deck {
            haw_ai_percentage,
            categories,
            token_ids,
            total_power,
        });
    }

    pub fn update_place(env: Env, owner: Address, categories: Vec<Category>, token_ids: Vec<TokenId>) {
        owner.require_auth();

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

        let haw_ai_percentage = ((total_power as f32) * (100 as f32)
            / (balance.total_deck_power as f32)
            * (1.0 + bonus as f32 / 100.0)) as u32;

        write_deck(env.clone(), owner.clone(), Deck {
            haw_ai_percentage,
            categories,
            token_ids,
            total_power,
        });
    }

    pub fn remove_place(env: Env, owner: Address) {
        owner.require_auth();

        let deck = read_deck(env.clone(), owner.clone());
        
        for i in 0..4 {
            let category = deck.categories.get(i).unwrap();
            let token_id = deck.token_ids.get(i).unwrap();
            let mut nft = read_nft(&env, owner.clone(), category.clone(), token_id.clone());
            nft.locked_by_action = Action::None;
            write_nft(&env.clone(), owner.clone(), category.clone(), token_id.clone(), nft);
        }

        let mut balance = read_balance(&env);
        balance.total_deck_power -= deck.total_power;
        write_balance(&env, &balance);

        remove_deck(env.clone(), owner.clone());
    }
}
