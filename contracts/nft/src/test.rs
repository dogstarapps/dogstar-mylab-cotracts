#![cfg(test)]
extern crate std;

use crate::nft_info::Card;
use crate::NFTClient;
use crate::{
    admin::Config,
    contract::NFT,
    metadata::CardMetadata,
    nft_info::{ Category, Currency, Action },
    storage_types::TokenId,
    actions::fight,
};
use soroban_sdk::token::StellarAssetClient;
use soroban_sdk::{log, testutils::Address as _, vec, Address, Env};
use soroban_sdk::{token::TokenClient, String};

fn create_nft<'a>(e: Env, admin: &Address, config: &Config) -> NFTClient<'a> {
    let nft: NFTClient = NFTClient::new(&e, &e.register_contract(None, NFT {}));
    nft.initialize(admin, config);
    nft
}

fn generate_config(e: &Env) -> Config {
    Config {
        xtar_token: Address::generate(e),
        oracle_contract_id: Address::generate(e),
        haw_ai_pot: Address::generate(e),
        withdrawable_percentage: 50,
        burnable_percentage: 50,
        how_ai_percentage: 50,
        terry_per_power: 100,
        terry_per_action: 10,
        stake_periods: vec![&e.clone(), 0, 200, 300],
        stake_interest_percentages: vec![&e.clone(), 1, 2, 3],
        power_action_fee: 1,
        burn_receive_percentage: 50,
        apy_alpha: 10,
    }
}

fn mint_token(e: &Env, token: Address, to: Address, amount: i128) {
    let token_admin_client = StellarAssetClient::new(&e, &token);
    token_admin_client.mint(&to, &amount);
}

fn create_metadata(e: &Env) -> CardMetadata {
    let metadata = CardMetadata {
        name: String::from_str(&e, "Tessa Trend"),
        base_uri: String::from_str(&e, ""),
        thumb_uri: String::from_str(&e, ""),
        description: String::from_str(&e, ""),
        initial_power: 1000,        // Set appropriate value
        max_power: 10000,           // Set appropriate value
        level: 1,                   // Set appropriate value
        category: Category::Leader, // Example category
        price_xtar: 100,            // Set appropriate value
        price_terry: 100,           // Set appropriate value
        token_id: 1,
    };
    metadata
}

#[test]
fn test_mint() {
    let e = Env::default();
    e.mock_all_auths();
    log!(&e, "test nft mint function");

    let admin = Address::generate(&e);
    let player1 = Address::generate(&e);
    let player2 = Address::generate(&e);

    // Generate config
    let mut config = generate_config(&e);

    let xtar_token = e.register_stellar_asset_contract(admin.clone());
    config.xtar_token = xtar_token.clone();
    let xtar_token_client = TokenClient::new(&e, &xtar_token);

    let nft = create_nft(e.clone(), &admin, &config);

    // Mint terry tokens to player1
    nft.mint_terry(&player1, &100000);
    assert_eq!(nft.terry_balance(&player1), 100000);

    // Mint xtar tokens to player2
    mint_token(&e, xtar_token.clone(), player2.clone(), 100000);
    assert_eq!(xtar_token_client.balance(&player2), 100000);

    let metadata = create_metadata(&e);
    nft.create_metadata(&metadata, &1);
    // Mint token 1 to player1
    assert!(nft.exists(&player1, &TokenId(1)) == false);
    nft.mint(&player1, &TokenId(1), &1, &Currency::Terry);
    assert!(nft.exists(&player1, &TokenId(1)) == true);

    // Mint token 2 to player2
    assert!(nft.exists(&player2, &TokenId(1)) == false);
    nft.mint(&player2, &TokenId(1), &1, &Currency::Xtar);
    assert!(nft.exists(&player2, &TokenId(1)) == true);
}

#[test]
fn test_add_power() {
    let e = Env::default();
    e.mock_all_auths();

    // initialize users
    let admin = Address::generate(&e);
    let player = Address::generate(&e);

    // Generate config
    let mut config = generate_config(&e);

    let xtar_token = e.register_stellar_asset_contract(admin.clone());
    config.xtar_token = xtar_token.clone();

    let nft = create_nft(e.clone(), &admin, &config);

    // Mint terry tokens to player
    nft.mint_terry(&player, &100000);
    assert_eq!(nft.terry_balance(&player), 100000);

    let metadata = create_metadata(&e);
    nft.create_metadata(&metadata, &1);
    // create player
    nft.create_user(&player);

    // mint
    nft.mint(&player, &TokenId(1), &1, &Currency::Terry);
    assert!(nft.exists(&player, &TokenId(1)) == true);

    // add power
    let amount: u32 = 10;
    nft.add_power_to_card(&player, &1, &amount);

    let user = nft.read_user(&player);
    let card: Card = nft.card(&player, &TokenId(1)).unwrap();
    assert_eq!(user.power, 90);

    assert_eq!(card.clone().power, 1010);
}

#[test]
fn test_stake() {
    let e = Env::default();
    e.mock_all_auths();
    
    std::println!("hello test stake");

    let admin = Address::generate(&e);
    let player = Address::generate(&e);
    // Generate config
    let mut config = generate_config(&e);

    let nft = create_nft(e.clone(), &admin, &config);
    
    let metadata = create_metadata(&e);
    nft.create_metadata(&metadata, &1);

    nft.create_user(&player);
    // Mint 100000 terry to player
    nft.mint_terry(&player, &100000);

    let user = nft.read_user(&player);
    
    assert_eq!(user.terry, 100000);


    // // Set user level
    // //nft.set_user_level(&user1.clone(), &1);

    // // Mint token 1 to user1
    assert!(nft.exists(&player,  &TokenId(1)) == false);

    nft.mint(&player, &TokenId(1), &1, &Currency::Terry);
    assert!(nft.exists(&player,  &TokenId(1)) == true);

    // let card_info = CardInfo::get_default_card(Category::Leader);

    // Stake
    let mut card = nft.card(&player,  &TokenId(1)).unwrap();
    
    let old_card_power = card.power.clone();

    let power_action_fee = config.power_action_fee * card.power / 100;
    nft.stake(&player, &Category::Leader, &TokenId(1), &0);
    card = nft.card(&player,  &TokenId(1)).unwrap();
    
    assert!(card.locked_by_action == Action::Stake);
    assert_eq!(card.power, 0);
    assert_eq!(power_action_fee, 10);

    let balance = nft.admin_balance();
    assert!(balance.haw_ai_power == power_action_fee);

    let stake = nft.read_stake(&player, &Category::Leader, &TokenId(1));
    assert_eq!(stake.power, 1000 - power_action_fee);

    // Unstake
    nft.unstake(&player, &Category::Leader, &TokenId(1));

    card = nft.card(&player, &TokenId(1)).unwrap();
    assert_eq!(card.power, old_card_power - power_action_fee);
    // assert!(nft.config().terry_token == config.terry_token);
    
}

#[test]
fn test_fight() {
    let e = Env::default();
    e.mock_all_auths();

    let admin1 = Address::generate(&e);
    let player = Address::generate(&e);

    // Generate config
    let mut config = generate_config(&e);

    let nft = create_nft(e.clone(), &admin1, &config);

    // Set user level
    //nft.set_user_level(&player.clone(), &1);

    let metadata = create_metadata(&e);
    nft.create_metadata(&metadata, &1);

    nft.create_user(&player);
    // Mint 100000 terry to player
    nft.mint_terry(&player, &100000);

    let user = nft.read_user(&player);
    
    assert_eq!(user.terry, 100000);

    // Mint token 1 to player
    assert!(nft.exists(&player,  &TokenId(1)) == false);
    nft.mint(&player,  &TokenId(1), &1, &Currency::Terry);
    assert!(nft.exists(&player,  &TokenId(1)) == true);

    // let card_info = CardInfo::get_default_card(Category::Leader);

    // Fight
    nft.open_position(
        &player,
        &Category::Leader,
        &TokenId(1),
        &fight::FightCurrency::BTC,
        &fight::SidePosition::Long,
        &120,
    );
    nft.close_position(&player.clone(), &Category::Leader, &TokenId(1));
}

#[test]
fn test_lending() {
    let e = Env::default();
    e.mock_all_auths();

    let admin1 = Address::generate(&e);
    let address1 = Address::generate(&e);
    let address2 = Address::generate(&e);

    // Generate config
    let mut config = generate_config(&e);
    let nft = create_nft(e.clone(), &admin1, &config);

    let metadata = create_metadata(&e);
    nft.create_metadata(&metadata, &1);

    // Mint terry tokens to address1 & address2

    nft.create_user(&address1);
    // Mint 100000 terry to user1
    nft.mint_terry(&address1, &100000);

    let user1 = nft.read_user(&address1);
    
    assert_eq!(user1.terry, 100000);

    nft.create_user(&address2);
    // Mint 100000 terry to user1
    nft.mint_terry(&address2, &200000);

    let user2 = nft.read_user(&address2);
    
    assert_eq!(user2.terry, 200000);


    // Set user level
    //nft.set_user_level(&user1.clone(), &1);
    //nft.set_user_level(&user2.clone(), &1);

    // Mint token 1 to user1
    assert!(nft.exists(&address1, &TokenId(1)) == false);
    nft.mint(&address1, &TokenId(1), &1, &Currency::Terry);
    assert!(nft.exists(&address1, &TokenId(1)) == true);

    assert!(nft.exists(&address2, &TokenId(1)) == false);
    nft.mint(&address2, &TokenId(1), &1, &Currency::Terry);
    assert!(nft.exists(&address2, &TokenId(1)) == true);

    // Create a Lend token 1
    nft.lend(&address1, &Category::Resource, &TokenId(1), &100);
    nft.borrow(&address2, &Category::Resource, &TokenId(1), &70);
    nft.repay(&address2, &Category::Resource, &TokenId(1));
    nft.withdraw(&address1, &Category::Resource, &TokenId(1));
}

#[test]
fn test_deck() {
    let e = Env::default();
    e.mock_all_auths();

    let admin1 = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);

    // Generate config
    let mut config = generate_config(&e);

    let nft = create_nft(e.clone(), &admin1, &config);

    // Set user level
    //nft.set_user_level(&user1.clone(), &1);
    //nft.set_user_level(&user2.clone(), &1);

    // set metadata
    let mut metadata_1 = create_metadata(&e);
    metadata_1.token_id = 1;
    metadata_1.category = Category::Leader;
    let mut metadata_2 = create_metadata(&e);
    metadata_2.token_id = 2;
    metadata_2.category = Category::Skill;
    let mut metadata_3 = create_metadata(&e);
    metadata_3.token_id = 3;
    metadata_3.category = Category::Resource;
    let mut metadata_4 = create_metadata(&e);
    metadata_4.token_id = 4;
    metadata_4.category = Category::Weapon;

    nft.create_metadata(&metadata_1, &1);
    nft.create_metadata(&metadata_2, &2);
    nft.create_metadata(&metadata_3, &3);
    nft.create_metadata(&metadata_4, &4);

    nft.create_user(&user1);
    // Mint 100000 terry to player
    nft.mint_terry(&user1, &100000);

    nft.create_user(&user2);
    // Mint 100000 terry to player
    nft.mint_terry(&user2, &100000);


    // Mint token 1,2,3,4 to user1
    assert!(nft.exists(&user1,  &TokenId(1)) == false);
    nft.mint(&user1,  &TokenId(1), &1, &Currency::Terry);
    assert!(nft.exists(&user1, &TokenId(1)) == true);

    nft.mint(&user1,  &TokenId(2), &1, &Currency::Terry);
    nft.mint(&user1,  &TokenId(3), &1, &Currency::Terry);
    nft.mint(&user1,  &TokenId(4), &1, &Currency::Terry);

    assert!(nft.exists(&user2,  &TokenId(1)) == false);
    nft.mint(&user2,  &TokenId(1), &1, &Currency::Terry);
    assert!(nft.exists(&user2,  &TokenId(1)) == true);

    nft.mint(&user2,  &TokenId(2), &1, &Currency::Terry);
    nft.mint(&user2,  &TokenId(3), &1, &Currency::Terry);
    nft.mint(&user2,  &TokenId(4), &1, &Currency::Terry);

    nft.place(&user1, &TokenId(1));
    nft.place(&user1, &TokenId(2));
    nft.place(&user1, &TokenId(3));
    nft.place(&user1, &TokenId(4));

    let mut deck1 = nft.read_deck(&user1);
    // let balance = read_balance(&e);
    log!(&e, "bonus of deck1 {}", deck1.bonus);
    assert_eq!(deck1.bonus, 25);
    assert_eq!(deck1.token_ids.len(), 4);
    assert_eq!(deck1.total_power, 4000);
    
    let mut balance = nft.admin_balance();
    assert_eq!(balance.total_deck_power, 4000);

    nft.remove_place(&user1, &TokenId(1));
    nft.remove_place(&user1, &TokenId(2));

    deck1 = nft.read_deck(&user1);
    assert_eq!(deck1.bonus, 0);
    assert_eq!(deck1.total_power, 0);
    assert_eq!(deck1.token_ids.len(), 3);
    
}
