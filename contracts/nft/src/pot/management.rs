use crate::actions::deck::read_deck;
use crate::event::*;
use crate::storage_types::{
    DataKey, Deck, DogstarBalance, PendingReward, PlayerReward, PotBalance, PotSnapshot,
};
use soroban_sdk::{Address, Env, Vec};

const DAY_IN_LEDGERS: u32 = 17280;
const POT_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
const POT_LIFETIME_THRESHOLD: u32 = POT_BUMP_AMOUNT - DAY_IN_LEDGERS;

/// Calculates effective power by applying the deck bonus to base power.
pub fn calculate_effective_power(base_power: u32, deck_bonus: u32) -> u32 {
    base_power * (100 + deck_bonus) / 100
}

// Pot Balance Management
pub fn read_pot_balance(env: &Env) -> PotBalance {
    env.storage()
        .persistent()
        .get(&DataKey::PotBalance)
        .unwrap_or(PotBalance {
            accumulated_terry: 0,
            accumulated_power: 0,
            accumulated_xtar: 0,
            last_opening_round: 0,
            total_openings: 0,
            last_updated: env.ledger().timestamp(),
        })
}

pub fn write_pot_balance(env: &Env, balance: &PotBalance) {
    env.storage()
        .persistent()
        .set(&DataKey::PotBalance, balance);
}

pub fn read_dogstar_balance(env: &Env) -> DogstarBalance {
    env.storage()
        .persistent()
        .get(&DataKey::DogstarBalance)
        .unwrap_or(DogstarBalance {
            terry: 0,
            power: 0,
            xtar: 0,
        })
}

pub fn write_dogstar_balance(env: &Env, balance: &DogstarBalance) {
    env.storage()
        .persistent()
        .set(&DataKey::DogstarBalance, balance);
}

// Snapshot Management
pub fn write_pot_snapshot(env: &Env, round: u32, snapshot: &PotSnapshot) {
    let key = DataKey::OpeningSnapshot(round);

    env.storage().persistent().set(&key, snapshot);
}

pub fn read_pot_snapshot(env: &Env, round: u32) -> Option<PotSnapshot> {
    env.storage()
        .persistent()
        .get(&DataKey::OpeningSnapshot(round))
}

pub fn write_player_reward(env: &Env, round: u32, player: &Address, reward: &PlayerReward) {
    let key = DataKey::PlayerShare(round, player.clone());

    env.storage().persistent().set(&key, reward);
}

pub fn read_player_reward(env: &Env, round: u32, player: &Address) -> Option<PlayerReward> {
    env.storage()
        .persistent()
        .get(&DataKey::PlayerShare(round, player.clone()))
}

pub fn write_pending_reward(env: &Env, round: u32, player: &Address, reward: &PendingReward) {
    let key = DataKey::PendingReward(round, player.clone());
    env.storage()
        .persistent()
        .extend_ttl(&key, POT_LIFETIME_THRESHOLD, POT_BUMP_AMOUNT);
    env.storage().persistent().set(&key, reward);
}

pub fn read_pending_reward(env: &Env, round: u32, player: &Address) -> Option<PendingReward> {
    env.storage()
        .persistent()
        .get(&DataKey::PendingReward(round, player.clone()))
}

pub fn get_current_round(env: &Env) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::CurrentRound)
        .unwrap_or(0)
}

pub fn set_current_round(env: &Env, round: u32) {
    env.storage()
        .persistent()
        .set(&DataKey::CurrentRound, &round);
}

pub fn get_all_rounds(env: &Env) -> Vec<u32> {
    env.storage()
        .persistent()
        .get(&DataKey::AllRounds)
        .unwrap_or_else(|| Vec::new(env))
}

pub fn add_round(env: &Env, round: u32) {
    let mut rounds = get_all_rounds(env);
    rounds.push_back(round);

    env.storage().persistent().set(&DataKey::AllRounds, &rounds);
}

pub fn get_eligible_players(env: &Env) -> Vec<Address> {
    let mut eligible_players = Vec::new(env);
    let decks = env
        .storage()
        .persistent()
        .get::<DataKey, Vec<Deck>>(&DataKey::Decks)
        .unwrap_or(Vec::new(env));

    for deck in decks.iter() {
        if deck.token_ids.len() == 4 {
            eligible_players.push_back(deck.owner);
        }
    }

    eligible_players
}


pub fn get_eligible_players_with_shares(env: &Env, round: u32) -> Vec<(Address, u32)> {
    let players = get_eligible_players(env);
    let mut total_effective_power: u32 = 0;
    let mut player_powers = Vec::new(env);

    for player in players.iter() {
        let deck = read_deck(env.clone(), player.clone());
        if deck.token_ids.len() == 4 {
            let effective_power = calculate_effective_power(deck.total_power, deck.bonus);
            total_effective_power += effective_power;
            player_powers.push_back((player, effective_power));
        }
    }

    let mut result = Vec::new(env);
    for (player, effective_power) in player_powers.iter() {
        let share_percentage = if total_effective_power > 0 {
            (effective_power * 10000) / total_effective_power // Basis points
        } else {
            0
        };
        result.push_back((player, share_percentage));
    }
    result
}

pub fn calculate_player_shares(env: &Env, round: u32) {
    let players = get_eligible_players(env);
    let mut total_effective_power: u32 = 0;
    let mut player_powers = Vec::new(env);

    for player in players.iter() {
        let deck = read_deck(env.clone(), player.clone());
        if deck.token_ids.len() == 4 {
            let effective_power = calculate_effective_power(deck.total_power, deck.bonus);
            total_effective_power += effective_power;
            player_powers.push_back((player, effective_power, deck.bonus, deck.deck_categories));
        }
    }

    for (player, effective_power, deck_bonus, deck_categories) in player_powers.iter() {
        let share_percentage = if total_effective_power > 0 {
            (effective_power * 10000) / total_effective_power // Basis points
        } else {
            0
        };

        let reward = PlayerReward {
            share_percentage,
            effective_power,
            round_number: round,
            deck_bonus,
            deck_categories,
        };

        write_player_reward(env, round, &player, &reward);
        emit_share_calculated(env, &player, &reward);
    }

    if let Some(mut snapshot) = read_pot_snapshot(env, round) {
        snapshot.total_participants = player_powers.len();
        snapshot.total_effective_power = total_effective_power;
        write_pot_snapshot(env, round, &snapshot);
    }
}
