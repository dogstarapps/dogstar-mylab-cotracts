use crate::*;
use admin::{read_balance, write_balance};
use nft_info::{read_nft, write_nft, Action, Category};
use user_info::read_user;
use metadata::read_metadata;
use soroban_sdk::{contracttype, vec, Address, Env, Vec, log};
use storage_types::{DataKey, TokenId, BALANCE_BUMP_AMOUNT, BALANCE_LIFETIME_THRESHOLD};

#[contracttype]
#[derive(Clone, Eq, PartialEq)]
pub struct Deck {
    pub owner: Address,
    // pub categories: Vec<Category>,
    pub token_ids: Vec<TokenId>,
    pub total_power: u32,
    pub haw_ai_percentage: u32,
    pub bonus: u32,
}

pub fn write_deck(env: Env, fee_payer: Address, deck: Deck) {
    let owner = read_user(&env, fee_payer).owner;

    let key = DataKey::Deck(owner.clone());
    env.storage().persistent().set(&key, &deck);
    #[cfg(not(test))]
    {
        env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
    }
    
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

    let key = DataKey::Deck(owner.clone());
    let new_deck = Deck {
        owner: owner.clone(),
        haw_ai_percentage: 0,
        total_power: 0,
        bonus: 0,
        token_ids: Vec::new(&env),
    };
    if !env.storage().persistent().has(&key) {
        env.storage().persistent().set(&key, &new_deck);
    }
    #[cfg(not(test))]
    {
        env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
    }
    
    env.storage().persistent().get(&key).unwrap_or(new_deck.clone())
}

pub fn place(env: Env, fee_payer: Address, token_id: TokenId) {
    // fetch deck
    let mut deck = read_deck(env.clone(), fee_payer.clone());

    assert!(deck.token_ids.len() < 4, "Decks are exceed!");

    // read and update nft action
    let mut nft = read_nft(&env, fee_payer.clone(), token_id.clone()).unwrap();

    assert!(
        nft.locked_by_action == Action::None,
        "Locked by other action"
    );

    nft.locked_by_action = Action::Deck;

    write_nft(
        &env,
        fee_payer.clone(),
        token_id.clone(),
        nft.clone()
    );
    let mut unique_categories = vec![&env.clone()];

    deck.token_ids.push_back(token_id.clone());
    let mut total_power = 0;
    if (deck.token_ids.len() == 4) {
        // check unique categories
        for id in deck.token_ids.iter() {
            let _nft = read_nft(&env, fee_payer.clone(), id.clone()).unwrap();
            let metadata = read_metadata(&env, id.clone().0);
            let category = metadata.category;

            total_power += _nft.power;

            if !unique_categories.contains(&category) {
                unique_categories.push_back(category.clone());
            }
        }
        // calculate bonus 
        let bonus = match unique_categories.len() {
            1 => 0,
            2 => 5, 
            3 => 10,
            4 => 25,
            _ => 0,
        };

        // update power balance
        let mut balance = read_balance(&env);
        balance.total_deck_power += total_power;

        write_balance(&env, &balance);

        // write deck

        deck.bonus = bonus;
        deck.total_power = total_power;

        // update haw ai percentages
        update_haw_ai_percentages(env.clone());
    }
    
    write_deck(
        env.clone(),
        fee_payer.clone(),
        deck,
    );
}

pub fn update_place(env: Env) {

}

pub fn remove_place(env: Env, fee_payer: Address, token_id: TokenId) {
    // fetch deck
    let mut deck = read_deck(env.clone(), fee_payer.clone());

    assert!(deck.token_ids.len() > 0, "Decks are null!");

    // read and update nft action
    let mut nft = read_nft(&env, fee_payer.clone(), token_id.clone()).unwrap();
    
    log!(&env, "deck token id length {}", deck.token_ids.len());

    // remove token id
    if let Some(index) = deck.token_ids.iter().position(|x| x == token_id.clone()) {
        deck.token_ids.remove(index.try_into().unwrap());
    }
    
    assert!(
        nft.locked_by_action == Action::Deck,
        "Not locked by Deck"
    );
    // update action and save it
    nft.locked_by_action = Action::None;

    write_nft(
        &env,
        fee_payer.clone(),
        token_id.clone(),
        nft.clone()
    );

    // update power balance, deduct as much as deck's power
    let mut balance = read_balance(&env);
    balance.total_deck_power -= deck.total_power.clone();

    // write_balance(&env, &balance);
    
    // write deck
    deck.total_power = 0;
    deck.bonus = 0;

    write_deck(
        env.clone(),
        fee_payer.clone(),
        deck,
    );
    // update haw ai percentages
    update_haw_ai_percentages(env);
}

pub fn remove_all_place(env: Env, fee_payer: Address) {
    let deck = read_deck(env.clone(), fee_payer.clone());

    // release all action, write nft
    for i in 0..4 {
        let token_id = deck.token_ids.get(i).unwrap();

        let mut nft = read_nft(&env, fee_payer.clone(), token_id.clone()).unwrap();
        nft.locked_by_action = Action::None;

        write_nft(
            &env.clone(),
            fee_payer.clone(),
            token_id.clone(),
            nft,
        );
    }

    // update balance
    let mut balance = read_balance(&env);
    balance.total_deck_power -= deck.total_power;
    write_balance(&env, &balance);

    // remove deck
    remove_deck(env.clone(), fee_payer.clone());

    // update haw ai percentage
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
