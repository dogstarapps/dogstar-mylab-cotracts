//! This contract demonstrates a sample implementation of the Soroban token
//! interface.

use crate::actions::read_deck;
use crate::actions::{
    burn, deck, fight, lending,
    lending::{Borrowing, Lending},
    stake, SidePosition,
};
use crate::admin::{
    add_level, has_administrator, read_administrator, read_balance, read_config, read_state,
    update_level, write_administrator, write_balance, write_config,
};
use crate::error::NFTError;
use crate::event::*;
use crate::metadata::{read_metadata, write_metadata, CardMetadata};
use crate::nft_info::{exists, read_nft, remove_nft, write_nft, Action, Card, Category, Currency};
use crate::pot::management::*;
use crate::pot::reward::*;
use crate::storage_types::*;
use crate::user_info::{
    add_card_to_owner, burn_terry, get_user_level, mint_terry, read_owner_card, read_user,
    write_user,
};

use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, token, Address, BytesN, Env, Symbol,log
};
use soroban_sdk::{vec, String, Vec};
use soroban_token_sdk::TokenUtils;

#[contract]
pub struct NFT;

#[contractimpl]
impl NFT {
    pub fn initialize(e: Env, admin: Address, config: Config) {
        // Check if the contract is already initialized
        if has_administrator(&e) {
            panic!("already initialized");
        }
        write_administrator(&e, &admin);
        write_config(&e, &config);
        write_balance(
            &e,
            &Balance {
                admin_power: 0,
                admin_terry: 0,
                haw_ai_power: 0,
                haw_ai_terry: 0,
                haw_ai_xtar: 0,
                total_deck_power: 0,
            },
        );

        let levels = vec![
            &e,
            Level {
                minimum_terry: 0,
                maximum_terry: 1000,
                name: String::from_str(&e, "PLASTICLOVER"),
            },
            Level {
                minimum_terry: 1001,
                maximum_terry: 5000,
                name: String::from_str(&e, "SHITCOINER"),
            },
            Level {
                minimum_terry: 5001,
                maximum_terry: 25000,
                name: String::from_str(&e, "MEMECE0"),
            },
            Level {
                minimum_terry: 25001,
                maximum_terry: 200000,
                name: String::from_str(&e, "CRYPTOBRO"),
            },
            Level {
                minimum_terry: 200001,
                maximum_terry: 500000,
                name: String::from_str(&e, "CHIEF"),
            },
            Level {
                minimum_terry: 500001,
                maximum_terry: 2000000,
                name: String::from_str(&e, "BOSS"),
            },
            Level {
                minimum_terry: 2000001,
                maximum_terry: 5000000,
                name: String::from_str(&e, "DIVINE"),
            },
            Level {
                minimum_terry: 5000001,
                maximum_terry: 10000000,
                name: String::from_str(&e, "LEGEND"),
            },
            Level {
                minimum_terry: 10000001,
                maximum_terry: 15000000,
                name: String::from_str(&e, "IMMORTAL"),
            },
            Level {
                minimum_terry: 15000001,
                maximum_terry: i128::MAX,
                name: String::from_str(&e, "Level 10"),
            },
        ];

        for (_, level) in levels.into_iter().enumerate() {
            add_level(&e, level);
        }

        // Emit initialization event
        e.events()
            .publish((Symbol::new(&e, "initialized"),), (admin,));
    }

    pub fn add_new_level(e: Env, level: Level) {
        let admin: Address = read_administrator(&e);
        admin.require_auth();
        add_level(&e, level);
    }

    pub fn update_level(e: Env, level_id: u32, level: Level) {
        let admin: Address = read_administrator(&e);
        admin.require_auth();
        update_level(&e, level_id, level);
    }

    pub fn mint_terry(e: Env, player: Address, amount: i128) {
        let admin = read_administrator(&e);
        admin.require_auth();
        mint_terry(&e, player, amount);
    }

    pub fn batch_mint_terry(e: Env, to_addresses: Vec<Address>, amounts: Vec<i128>) {
        let admin = read_administrator(&e);
        admin.require_auth();
        if to_addresses.len() != amounts.len() {
            panic!("Mismatched lengths of addresses and amounts");
        }
        for (to, amount) in to_addresses.iter().zip(amounts.iter()) {
            mint_terry(&e, to, amount);
        }
    }

    pub fn terry_balance(e: Env, player: Address) -> i128 {
        let user = read_user(&e, player);
        user.terry
    }

    pub fn mint(
        env: Env,
        user: Address,
        token_id: TokenId,
        card_level: u32,
        buy_currency: Currency,
    ) {
        user.require_auth();

        let user: User = read_user(&env, user.clone());
        let to: Address = user.owner.clone();
        let user_level = get_user_level(&env, to.clone());

        assert!(
            user_level >= card_level,
            "User level too low to mint this card"
        );
        assert!(
            !Self::exists(&env, to.clone(), token_id.clone()),
            "Token ID already exists"
        );

        let card_metadata = read_metadata(&env, token_id.clone().0);
        let nft = Card {
            power: card_metadata.initial_power,
            locked_by_action: Action::None,
        };
        write_nft(&env, to.clone(), token_id.clone(), nft.clone());

        add_card_to_owner(&env, token_id.clone(), to.clone()).unwrap();

        let config: Config = read_config(&env);
        let mut balance = read_balance(&env);

        // matches!(buy_currency, Currency::Terry)
        //     .then(|| {
        //         assert!(
        //             user.terry >= card_metadata.price_terry,
        //             "Not enough terry to mint this card"
        //         );
        //     })
        //     .unwrap_or_else(|| {
        //         assert!(
        //             user.power >= card_metadata.price_xtar as u32,
        //             "Not enough xtar to mint this card"
        //         );
        //     });

        if buy_currency == Currency::Terry {
            let amount = card_metadata.price_terry;
            let withdrawable_amount = (config.withdrawable_percentage as i128 * amount) / 100;
            let haw_ai_amount = amount - withdrawable_amount;
            burn_terry(&env, user.owner.clone(), amount);
            balance.admin_terry += withdrawable_amount;
            Self::accumulate_pot(env.clone(), haw_ai_amount, 0, 0);
        } else {
            let token = token::Client::new(&env, &config.xtar_token.clone());
            let burnable_amount =
                (config.burnable_percentage as i128 * card_metadata.price_xtar) / 100;
            let haw_ai_amount = card_metadata.price_xtar - burnable_amount;
            token.burn(&to.clone(), &burnable_amount);
            token.transfer(&to.clone(), &config.haw_ai_pot, &haw_ai_amount);
            balance.haw_ai_xtar += haw_ai_amount;
            Self::accumulate_pot(env.clone(), 0, 0, haw_ai_amount);
        };
        write_balance(&env, &balance);
    }

    pub fn transfer(env: Env, from: Address, to: Address, token_id: TokenId) {
        from.require_auth();
        let nft: Card = read_nft(&env, from.clone(), token_id.clone()).unwrap();
        remove_nft(&env, from.clone(), token_id.clone());
        write_nft(&env, to, token_id, nft);
    }

    pub fn burn(env: Env, user: Address, token_id: TokenId) {
        burn::burn(env, user, token_id)
    }

    pub fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        let admin: Address = read_administrator(&e);
        admin.require_auth();
        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    pub fn set_admin(e: Env, new_admin: Address) {
        let admin = read_administrator(&e);
        admin.require_auth();
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        write_administrator(&e, &new_admin);
        TokenUtils::new(&e).events().set_admin(admin, new_admin);
    }

    pub fn add_level(e: &Env, level: Level) -> u32 {
        add_level(e, level)
    }

    pub fn add_to_whitelist(e: &Env, members: Vec<Address>) {
        let admin = read_administrator(e);
        admin.require_auth();
        for member in members.iter() {
            e.storage()
                .persistent()
                .set(&DataKey::Whitelist(member.clone()), &true);
        }
    }

    pub fn remove_from_whitelist(e: &Env, members: Vec<Address>) {
        let admin = read_administrator(e);
        admin.require_auth();
        for member in members.iter() {
            e.storage()
                .persistent()
                .remove(&DataKey::Whitelist(member.clone()));
        }
    }

    pub fn card(env: &Env, owner: Address, token_id: TokenId) -> Option<Card> {
        read_nft(env, owner, token_id)
    }

    pub fn exists(env: &Env, owner: Address, token_id: TokenId) -> bool {
        exists(env, owner, token_id)
    }

    pub fn admin_balance(env: &Env) -> Balance {
        read_balance(&env)
    }

    pub fn admin_state(env: &Env) -> State {
        read_state(&env)
    }

    pub fn config(env: Env) -> Config {
        read_config(&env)
    }

    pub fn create_metadata(e: &Env, card: CardMetadata, id: u32) {
        let admin = read_administrator(&e);
        admin.require_auth();
        write_metadata(e, id, card);
    }

    pub fn get_card(e: &Env, id: u32) -> CardMetadata {
        read_metadata(e, id)
    }

    pub fn create_user(e: Env, address: Address) {
        let admin = read_administrator(&e);
        admin.require_auth();
        let user: User = User {
            owner: address.clone(),
            power: 100,
            terry: 0,
            total_history_terry: 0,
            level: 1,
        };
        write_user(&e, address, user);
    }

    pub fn get_all_cards(e: &Env) -> soroban_sdk::Vec<CardMetadata> {
        let mut all_cards = soroban_sdk::Vec::new(&e);
        let card_ids = e
            .storage()
            .persistent()
            .get::<DataKey, soroban_sdk::Vec<TokenId>>(&DataKey::AllCardIds)
            .unwrap_or(soroban_sdk::Vec::new(&e));
        for token_id in card_ids.iter() {
            let card_metadata = read_metadata(e, token_id.0);
            all_cards.push_back(card_metadata);
        }
        all_cards
    }

    pub fn get_player_cards_with_state(
        e: &Env,
        player: Address,
    ) -> soroban_sdk::Vec<(CardMetadata, Card)> {
        let mut player_cards = soroban_sdk::Vec::new(&e);
        let owned_card_ids = read_owner_card(e, player.clone());
        for token_id in owned_card_ids.iter() {
            let card_metadata = read_metadata(e, token_id.0);
            if let Some(card) = read_nft(e, player.clone(), token_id.clone()) {
                player_cards.push_back((card_metadata, card));
            }
        }
        player_cards
    }

    pub fn add_power_to_card(env: &Env, player: Address, token_id: u32, amount: u32) {
        let card = read_nft(env, player.clone(), TokenId(token_id)).unwrap();
        let new_card = Card {
            power: card.power + amount,
            locked_by_action: card.locked_by_action,
        };
        write_nft(env, player.clone(), TokenId(token_id), new_card);
        let mut user = read_user(env, player.clone());
        user.power = user.power - amount;
        write_user(env, player.clone(), user);
    }

    pub fn read_user(env: &Env, player: Address) -> User {
        read_user(env, player.clone())
    }
}

//Pot Management

#[contractimpl]
impl NFT {
    pub fn get_current_pot_state(env: Env) -> (PotBalance, DogstarBalance) {
        (read_pot_balance(&env), read_dogstar_balance(&env))
    }

    pub fn get_player_potential_reward(env: Env, player: Address) -> PendingReward {
        let current_round = get_current_round(&env);
        let balance = read_pot_balance(&env);
        let deck = read_deck(env.clone(), player.clone());
        let players = get_eligible_players(&env);
        let mut total_effective_power: u128 = 0;
        for p in players.iter() {
            let d = read_deck(env.clone(), p.clone());
            if d.token_ids.len() == 4 {
                total_effective_power += calculate_effective_power(d.total_power, d.bonus) as u128;
            }
        }
        const PRECISION: u128 = 10000;
        let effective_power = calculate_effective_power(deck.total_power, deck.bonus) as u128;
        let share = if total_effective_power > 0 {
            ((effective_power * PRECISION) / total_effective_power) as u128
        } else {
            0
        };
        PendingReward {
            round_number: current_round,
            terry_amount: (balance.accumulated_terry * share as i128) / PRECISION as i128,
            power_amount: (balance.accumulated_power as u128 * share / PRECISION) as u32,
            xtar_amount: (balance.accumulated_xtar * share as i128) / PRECISION as i128,
            status: RewardStatus::Pending,
        }
    }

    pub fn get_historical_snapshot(e: Env, round: u32) -> Option<PotSnapshot> {
        read_pot_snapshot(&e, round)
    }

    pub fn get_player_participation(env: Env, player: Address, round: u32) -> Option<PlayerReward> {
        read_player_reward(&env, round, &player)
    }

    pub fn get_pending_rewards(env: Env, player: Address) -> Vec<PendingReward> {
        let mut pending = Vec::new(&env);
        for round in get_all_rounds(&env).iter() {
            if let Some(reward) = read_pending_reward(&env, round, &player) {
                if reward.status != RewardStatus::Claimed {
                    pending.push_back(reward);
                }
            }
        }
        pending
    }

    pub fn accumulate_pot(env: Env, terry: i128, power: u32, xtar: i128) {
        // let admin = read_administrator(&env);
        // admin.require_auth(); // commented out for testing
        let config = read_config(&env);
        let mut pot_balance = read_pot_balance(&env);
        let mut dogstar_balance = read_dogstar_balance(&env);
        let fee_percentage = config.dogstar_fee_percentage;
        let terry_fee = (terry * fee_percentage as i128) / 10000;
        let power_fee = (power * fee_percentage) / 10000;
        let xtar_fee = (xtar * fee_percentage as i128) / 10000;
        pot_balance.accumulated_terry += terry - terry_fee;
        pot_balance.accumulated_power += power - power_fee;
        pot_balance.accumulated_xtar += xtar - xtar_fee;
        pot_balance.last_updated = env.ledger().timestamp();
        dogstar_balance.terry += terry_fee;
        dogstar_balance.power += power_fee;
        dogstar_balance.xtar += xtar_fee;
        write_pot_balance(&env, &pot_balance);
        write_dogstar_balance(&env, &dogstar_balance);
        if terry_fee > 0 || power_fee > 0 || xtar_fee > 0 {
            emit_dogstar_fee_accumulated(&env, terry_fee, power_fee, xtar_fee, fee_percentage);
        }
    }

    pub fn withdraw_dogstar_fees(env: Env) {
        let config = read_config(&env);
        let dogstar_address = config.dogstar_address.clone();
        // dogstar_address.require_auth();
        let mut balance = read_dogstar_balance(&env);
        let terry = balance.terry;
        let power = balance.power;
        let xtar = balance.xtar;
        if terry > 0 {
            let mut user = read_user(&env, dogstar_address.clone());
            user.terry += terry;
            write_user(&env, dogstar_address.clone(), user);
            balance.terry = 0;
        }
        if power > 0 {
            let mut user = read_user(&env, dogstar_address.clone());
            user.power += power;
            write_user(&env, dogstar_address.clone(), user);
            balance.power = 0;
        }
        if xtar > 0 {
            let token = token::Client::new(&env, &config.xtar_token);
            token.transfer(&env.current_contract_address(), &dogstar_address, &xtar);
            balance.xtar = 0;
        }
        write_dogstar_balance(&env, &balance);
        emit_dogstar_fee_withdrawn(&env, &dogstar_address, terry, power, xtar);
    }

    pub fn open_pot(env: Env, round: u32) -> Result<(), NFTError> {
        let admin = read_administrator(&env);
        admin.require_auth();
        let current_round = get_current_round(&env);
        if round <= current_round {
            return Err(NFTError::RoundAlreadyProcessed);
        }
        let balance = read_pot_balance(&env);
        let snapshot = PotSnapshot {
            round_number: round,
            total_terry: balance.accumulated_terry,
            total_power: balance.accumulated_power,
            total_xtar: balance.accumulated_xtar,
            timestamp: env.ledger().timestamp(),
            total_participants: 0,
            total_effective_power: 0,
        };
        write_pot_snapshot(&env, round, &snapshot);
        emit_pot_opened(&env, round, &snapshot);
        calculate_player_shares(&env, round);
        set_current_round(&env, round);
        add_round(&env, round);
        write_pot_balance(
            &env,
            &PotBalance {
                accumulated_terry: 0,
                accumulated_power: 0,
                accumulated_xtar: 0,
                last_opening_round: round,
                total_openings: balance.total_openings + 1,
                last_updated: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    pub fn claim_all_pending_rewards(env: Env, player: Address) {
        claim_all_pending_rewards(env, player);
    }

    pub fn update_dogstar_fee_percentage(env: Env, fee_percentage: u32) {
        // let admin = read_administrator(&env);
        // admin.require_auth();

        let mut config = read_config(&env);
        let old_fee = config.dogstar_fee_percentage;
        config.dogstar_fee_percentage = fee_percentage;
        write_config(&env, &config);
        emit_dogstar_fee_percentage_updated(&env, old_fee, fee_percentage);
    }

    pub fn contribute_to_pot(env: Env, terry: i128, power: u32, xtar: i128) {
        let admin = read_administrator(&env);
        admin.require_auth();
        assert!(
            terry >= 0 && xtar >= 0,
            "Negative contributions not allowed"
        );
        Self::accumulate_pot(env, terry, power, xtar);
    }

    pub fn get_eligible_players(env: Env) -> Vec<Address> {
        get_eligible_players(&env)
    }

    pub fn get_eligible_players_with_shares(env: Env) -> Vec<(Address, u32)> {
        get_eligible_players_with_shares(&env)
    }

    pub fn get_all_rounds(env: Env) -> Vec<u32> {
        get_all_rounds(&env)
    }
}

// Stake, Fight, Lend & Borrow, Deck sections unchanged
#[contractimpl]
impl NFT {
    pub fn stake(
        env: Env,
        user: Address,
        category: Category,
        token_id: TokenId,
        period_index: u32,
    ) {
        stake::stake(env, user, category, token_id, period_index)
    }

    pub fn increase_stake_power(
        env: Env,
        user: Address,
        category: Category,
        token_id: TokenId,
        increase_power: u32,
    ) {
        stake::increase_stake_power(env, user, category, token_id, increase_power)
    }

    pub fn unstake(env: Env, user: Address, category: Category, token_id: TokenId) {
        stake::unstake(env, user, category, token_id)
    }

    pub fn read_stake(
        env: &Env,
        user: Address,
        category: Category,
        token_id: TokenId,
    ) -> stake::Stake {
        stake::read_stake(env, user, category, token_id)
    }

    pub fn read_stakes(env: Env) -> Vec<stake::Stake> {
        stake::read_stakes(env)
    }
}

#[contractimpl]
impl NFT {
    pub fn open_position(
        env: Env,
        owner: Address,
        category: Category,
        token_id: TokenId,
        currency: fight::FightCurrency,
        side_position: SidePosition,
        leverage: u32,
        power_staked: u32,
    ) {
        fight::open_position(
            env,
            owner,
            category,
            token_id,
            currency,
            side_position,
            leverage,
            power_staked,
        )
    }

    pub fn close_position(env: Env, owner: Address, category: Category, token_id: TokenId) {
        fight::close_position(env, owner, category, token_id)
    }

    pub fn currency_price(env: Env, oracle_contract_id: Address) -> i128 {
        fight::get_currency_price(env, oracle_contract_id, fight::FightCurrency::BTC)
    }

    pub fn read_fight(
        env: Env,
        user: Address,
        category: Category,
        token_id: TokenId,
    ) -> fight::Fight {
        fight::read_fight(env, user, category, token_id)
    }

    pub fn read_fights(env: Env) -> Vec<fight::Fight> {
        fight::read_fights(env)
    }

    pub fn check_liquidation(env: Env, user: Address, category: Category, token_id: TokenId) {
        fight::check_liquidation(env, user, category, token_id)
    }
}

#[contractimpl]
impl NFT {
    pub fn lend(env: Env, lender: Address, category: Category, token_id: TokenId, power: u32) {
        lending::lend(env, lender, category, token_id, power)
    }

    pub fn borrow(env: Env, borrower: Address, category: Category, token_id: TokenId, power: u32) {
        lending::borrow(env, borrower, category, token_id, power)
    }

    pub fn repay(env: Env, borrower: Address, category: Category, token_id: TokenId) {
        lending::repay(env, borrower, category, token_id)
    }

    pub fn withdraw(env: Env, lender: Address, category: Category, token_id: TokenId) {
        lending::withdraw(env, lender, category, token_id)
    }

    pub fn get_current_apy(env: Env) -> u64 {
        lending::get_current_apy(env)
    }

    pub fn read_lending(
        env: Env,
        player: Address,
        category: Category,
        token_id: TokenId,
    ) -> lending::Lending {
        lending::read_lending(env, player, category, token_id)
    }

    pub fn read_borrowing(
        env: Env,
        player: Address,
        category: Category,
        token_id: TokenId,
    ) -> lending::Borrowing {
        player.require_auth()
        lending::read_borrowing(env, player, category, token_id)
    }

    pub fn read_borrowings(env: Env) -> Vec<Borrowing> {
        lending::read_borrowings(env)
    }

    pub fn read_lendings(env: Env) -> Vec<Lending> {
        lending::read_lendings(env)
    }
}

#[contractimpl]
impl NFT {
    pub fn place(env: Env, owner: Address, token_id: TokenId) {
        deck::place(env, owner, token_id);
    }

    pub fn replace(env: Env, owner: Address, prev_token_id: TokenId, token_id: TokenId) {
        deck::replace(env, owner, prev_token_id, token_id);
    }

    pub fn remove_place(env: Env, owner: Address, token_id: TokenId) {
        deck::remove_place(env, owner, token_id)
    }

    pub fn read_deck(env: Env, owner: Address) -> Deck {
        deck::read_deck(env, owner)
    }
}
