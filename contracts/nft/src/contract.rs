//! This contract demonstrates a sample implementation of the Soroban token
//! interface.

use crate::actions::{burn, deck, fight, lending, stake, Deck, SidePosition};
use crate::admin::{
    add_level, has_administrator, read_administrator, read_balance, read_config, update_level,
    write_administrator, write_balance, write_config, Balance, Config,
};
use crate::metadata::{read_metadata, write_metadata, CardMetadata};
use crate::nft_info::{exists, read_nft, remove_nft, write_nft, Action, Card, Category, Currency};
use crate::storage_types::{
    DataKey, Level, TokenId, INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD,
};
use crate::user_info::{
    add_card_to_owner, burn_terry, get_user_level, mint_terry, read_owner_card, read_user,
    write_user, User,
};
use soroban_sdk::{contract, contractimpl, token, Address, BytesN, Env};
use soroban_sdk::{vec, String, Vec};
use soroban_token_sdk::TokenUtils;

#[contract]
pub struct NFT;

#[contractimpl]
impl NFT {
    pub fn initialize(e: Env, admin: Address, config: Config) {
        if has_administrator(&e) {
            panic!("already initialized")
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
            &e.clone(),
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
        fee_payer: Address,
        token_id: TokenId,
        card_level: u32,
        buy_currency: Currency,
    ) {
        fee_payer.require_auth();

        let user: User = read_user(&env, fee_payer.clone());
        let to: Address = user.owner;
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

        add_card_to_owner(&env, token_id.clone(), to.clone());
        // // puchase by currency
        let config: Config = read_config(&env);
        let mut balance = read_balance(&env);

        if buy_currency == Currency::Terry {
            let amount = card_metadata.price_terry;
            let withdrawable_amount = config.withdrawable_percentage as i128 * amount / 100;
            let haw_ai_amount = amount - withdrawable_amount;
            // burn terry
            burn_terry(&env, fee_payer.clone(), amount);
            // mint terry to admin & haw ai pot
            balance.admin_terry += withdrawable_amount;
            balance.haw_ai_terry += haw_ai_amount;
        } else {
            let token = token::Client::new(&env, &config.xtar_token.clone());
            let burnable_amount =
                (config.burnable_percentage as i128) * card_metadata.price_xtar / 100;
            let haw_ai_amount = card_metadata.price_terry - burnable_amount;
            // 50% of xtar price to burn and 50% to the haw ai pot
            token.burn(&to.clone(), &burnable_amount);
            token.transfer(&to.clone(), &config.haw_ai_pot, &haw_ai_amount);
            balance.haw_ai_xtar += haw_ai_amount;
        };

        write_balance(&env, &balance);
    }

    pub fn transfer(env: Env, from: Address, to: Address, token_id: TokenId) {
        from.require_auth();
        let nft: Card = read_nft(&env, from.clone(), token_id.clone()).unwrap();
        remove_nft(&env, from.clone(), token_id.clone());
        write_nft(&env, to, token_id, nft);
    }

    pub fn burn(env: Env, fee_payer: Address, token_id: TokenId) {
        burn::burn(env, fee_payer, token_id)
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

    pub fn config(env: Env) -> Config {
        read_config(&env)
    }

    pub fn create_metadata(e: &Env, card: CardMetadata, id: u32) {
        let admin = read_administrator(&e);
        admin.require_auth();
        write_metadata(e, id, card);
    }

    pub fn get_card(e: &Env, id: u32) -> CardMetadata {
        //let key = DataKey::Metadata(TokenId(id));
        //e.storage().instance().get(&key).unwrap()
        read_metadata(e, id)
    }

    pub fn create_user(e: Env, fee_payer: Address, owner: Address) {
        //it should be admin
        let user: User = User {
            owner,
            power: 100,
            terry: 0,
        };
        write_user(&e, fee_payer, user);
    }

    pub fn get_all_cards(e: &Env) -> soroban_sdk::Vec<CardMetadata> {
        let mut all_cards = soroban_sdk::Vec::new(&e);

        // Recuperamos todos los identificadores de cartas
        let card_ids = e
            .storage()
            .persistent()
            .get::<DataKey, soroban_sdk::Vec<TokenId>>(&DataKey::AllCardIds)
            .unwrap_or(soroban_sdk::Vec::new(&e));

        // Iteramos sobre cada identificador de carta y recuperamos la metadata
        for token_id in card_ids.iter() {
            let card_metadata = read_metadata(e, token_id.0);
            all_cards.push_back(card_metadata); // Usamos push_back en lugar de push
        }

        all_cards
    }

    pub fn get_player_cards_with_state(
        e: &Env,
        player: Address,
    ) -> soroban_sdk::Vec<(CardMetadata, Card)> {
        let mut player_cards = soroban_sdk::Vec::new(&e);

        // Recuperamos las cartas del jugador
        let owned_card_ids = read_owner_card(e, player.clone());

        // Iteramos sobre las cartas del jugador
        for token_id in owned_card_ids.iter() {
            let card_metadata = read_metadata(e, token_id.0); // Recupera la metadata
                                                              // Usamos `if let` para manejar la opción de la carta
            if let Some(card) = read_nft(e, player.clone(), token_id.clone()) {
                // Si la carta existe, la agregamos
                player_cards.push_back((card_metadata, card));
            }
        }

        player_cards
    }

    pub fn add_power_to_card(e: &Env, player: Address, token_id: u32, amount: u32) {
        let card = read_nft(e, player.clone(), TokenId(token_id)).unwrap();

        let new_card = Card {
            power: card.power + amount,
            locked_by_action: card.locked_by_action,
        };
        write_nft(e, player.clone(), TokenId(token_id), new_card);

        let mut user = read_user(e, player.clone());

        user.power = user.power - amount;

        write_user(e, player.clone(), user);
    }

    pub fn read_user(e: &Env, player: Address) -> User {
        read_user(e, player.clone())
    }
}

// Stake
#[contractimpl]
impl NFT {
    pub fn stake(
        env: Env,
        fee_payer: Address,
        category: Category,
        token_id: TokenId,
        stake_power: u32,
        period_index: u32,
    ) {
        stake::stake(
            env,
            fee_payer,
            category,
            token_id,
            stake_power,
            period_index,
        )
    }

    pub fn increase_stake_power(
        env: Env,
        fee_payer: Address,
        category: Category,
        token_id: TokenId,
        increase_power: u32,
    ) {
        stake::increase_stake_power(env, fee_payer, category, token_id, increase_power)
    }

    pub fn unstake(env: Env, fee_payer: Address, category: Category, token_id: TokenId) {
        stake::unstake(env, fee_payer, category, token_id)
    }

    pub fn read_stake(
        env: &Env,
        fee_payer: Address,
        category: Category,
        token_id: TokenId,
    ) -> stake::Stake {
        stake::read_stake(env, fee_payer, category, token_id)
    }
}

// Fight
#[contractimpl]
impl NFT {
    pub fn open_position(
        env: Env,
        owner: Address,
        category: Category,
        token_id: TokenId,
        currency: fight::Currency,
        side_position: SidePosition,
        leverage: u32,
    ) {
        fight::open_position(
            env,
            owner,
            category,
            token_id,
            currency,
            side_position,
            leverage,
        )
    }

    pub fn close_position(env: Env, owner: Address, category: Category, token_id: TokenId) {
        fight::close_position(env, owner, category, token_id)
    }

    pub fn currency_price(env: Env, oracle_contract_id: Address) -> i128 {
        fight::get_currency_price(env, oracle_contract_id, fight::Currency::BTC)
    }
}

// Lend & Borrow
#[contractimpl]
impl NFT {
    pub fn lend(
        env: Env,
        owner: Address,
        category: Category,
        token_id: TokenId,
        power: u32,
        interest_rate: u32,
        duration: u32,
    ) {
        lending::lend(
            env,
            owner,
            category,
            token_id,
            power,
            interest_rate,
            duration,
        )
    }

    pub fn borrow(
        env: Env,
        borrower: Address,
        lender: Address,
        category: Category,
        token_id: TokenId,
        collateral_category: Category,
        collateral_token_id: TokenId,
    ) {
        lending::borrow(
            env,
            borrower,
            lender,
            category,
            token_id,
            collateral_category,
            collateral_token_id,
        )
    }

    pub fn repay(
        env: Env,
        borrower: Address,
        lender: Address,
        category: Category,
        token_id: TokenId,
    ) {
        lending::repay(env, borrower, lender, category, token_id)
    }

    pub fn withdraw(env: Env, lender: Address, category: Category, token_id: TokenId) {
        lending::withdraw(env, lender, category, token_id)
    }
}

// Deck
#[contractimpl]
impl NFT {
    pub fn place(env: Env, owner: Address, categories: Vec<Category>, token_ids: Vec<TokenId>) {
        deck::place(env, owner, categories, token_ids)
    }

    pub fn update_place(
        env: Env,
        owner: Address,
        categories: Vec<Category>,
        token_ids: Vec<TokenId>,
    ) {
        deck::update_place(env, owner, categories, token_ids)
    }

    pub fn remove_place(env: Env, owner: Address) {
        deck::remove_deck(env, owner)
    }

    pub fn read_deck(env: Env, owner: Address) -> Deck {
        deck::read_deck(env, owner)
    }
}
