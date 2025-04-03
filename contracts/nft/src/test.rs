#![cfg(test)]
extern crate std;

use log::{ info, warn };
use env_logger;

use crate::{
    actions::{fight, SidePosition}, admin::Config, contract::NFT, metadata::{write_metadata, CardMetadata}, nft_info::{write_nft, Action, CardInfo, Category, Currency}, storage_types::TokenId, user_info::read_user, NFTClient
};
use soroban_sdk::{events, token::{StellarAssetClient, TokenClient}, String};
use soroban_sdk::{testutils::Address as _, vec, Address, Env, log};
use crate::nft_info::Card;

use crate::{ admin::{ transfer_terry, read_balance } };

fn create_nft<'a>(e: Env, admin: &Address, config: &Config) -> NFTClient<'a> {
    let nft: NFTClient = NFTClient::new(&e, &e.register_contract(None, NFT {}));
    nft.initialize(admin, config);

    nft
}


fn generate_config(e: &Env) -> Config {
    Config {
        terry_token: Address::generate(e),
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

    let admin1 = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);

    // Generate config
    let mut config = generate_config(&e);

    let terry_token = e.register_stellar_asset_contract(admin1.clone());
    config.terry_token = terry_token.clone();
    let xtar_token = e.register_stellar_asset_contract(admin1.clone());
    config.xtar_token = xtar_token.clone();
    let terry_token_client = TokenClient::new(&e, &terry_token);
    let xtar_token_client = TokenClient::new(&e, &xtar_token);

    let nft = create_nft(e.clone(), &admin1, &config);

    // Mint terry tokens to user1
    mint_token(&e, config.terry_token.clone(), user1.clone(), 100000);
    assert_eq!(terry_token_client.balance(&user1), 100000);

    // Mint xtar tokens to user2
    mint_token(&e, config.xtar_token.clone(), user2.clone(), 100000);
    assert_eq!(xtar_token_client.balance(&user2), 100000);

    // Set user level
    // nft.set_user_level(&user1.clone(), &1);
    // nft.set_user_level(&user2.clone(), &1);


    let metadata = create_metadata(&e);
    nft.create_metadata(&metadata, &1);
    // Mint token 1 to user1
    assert!(nft.exists(&user1,  &TokenId(1)) == false);
    nft.mint(&user1,  &TokenId(1), &1, &Currency::Terry);
    assert!(nft.exists(&user1,  &TokenId(1)) == true);

    let player_nft: soroban_sdk::Vec<(CardMetadata, crate::nft_info::Card)> = nft.get_player_cards_with_state(&user1);

    assert_eq!(player_nft.len(), 1);
    assert_eq!(
        player_nft.get(0).unwrap().0.name,
        String::from_str(&e, "Tessa Trend")
    );

    let card = nft.card(&user1, &TokenId(1)).unwrap();
    let card_info = CardInfo::get_default_card(Category::Leader);

    assert!( card.power == card_info.initial_power);

    assert_eq!(
        terry_token_client.balance(&user1),
        100000 - card_info.price_terry
    );

    let withdrawable_amount = config.withdrawable_percentage as i128 * card_info.price_terry / 100;
    assert_eq!(terry_token_client.balance(&admin1), withdrawable_amount);
    assert_eq!(
        terry_token_client.balance(&config.haw_ai_pot),
        card_info.price_terry - withdrawable_amount
    );

    let balance = nft.admin_balance();
    assert_eq!(balance.admin_terry, terry_token_client.balance(&admin1));
    assert_eq!(
        balance.haw_ai_terry,
        terry_token_client.balance(&config.haw_ai_pot)
    );

    // Mint token 2 to user1
    assert!(nft.exists(&user2,  &TokenId(1)) == false);
    nft.mint(&user2,  &TokenId(1), &1, &Currency::Xtar);
    assert!(nft.exists(&user2, &TokenId(1)) == true);

    let card = nft.card(&user2,  &TokenId(1)).unwrap();
    assert!( card.power == card_info.initial_power);

    assert_eq!(
        terry_token_client.balance(&user1),
        100000 - card_info.price_xtar
    );

    let burn_amount = config.burnable_percentage as i128 * card_info.price_xtar / 100;
    assert_eq!(
        terry_token_client.balance(&config.haw_ai_pot),
        card_info.price_xtar - burn_amount
    );

    let balance = nft.admin_balance();
    assert_eq!(
        balance.haw_ai_terry,
        terry_token_client.balance(&config.haw_ai_pot)
    );
}

#[test]
fn test_add_power() {
    let e = Env::default();
    e.mock_all_auths();
    // initialize users

    let admin1 = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);

    // create and initialize nft

    // Generate config
    let mut config = generate_config(&e);

    let terry_token = e.register_stellar_asset_contract(admin1.clone());
    config.terry_token = terry_token.clone();
    let xtar_token = e.register_stellar_asset_contract(admin1.clone());
    config.xtar_token = xtar_token.clone();
    let terry_token_client = TokenClient::new(&e, &terry_token);
    let xtar_token_client = TokenClient::new(&e, &xtar_token);

    let nft = create_nft(e.clone(), &admin1, &config);

    // Mint terry tokens to user1
    mint_token(&e, config.terry_token.clone(), user1.clone(), 100000);
    assert_eq!(terry_token_client.balance(&user1), 100000);

    // Mint xtar tokens to user2
    mint_token(&e, config.xtar_token.clone(), user2.clone(), 100000);
    assert_eq!(xtar_token_client.balance(&user2), 100000);

    let metadata = create_metadata(&e);
    nft.create_metadata(&metadata, &1);
    // create user
    nft.create_user(&user1, &user1);

    // mint
    nft.mint(&user1, &TokenId(1), &1, &Currency::Terry);

    assert!(nft.exists(&user1, &TokenId(1)) == true);
    // add power

    let amount: u32 = 10;
    nft.add_power_to_card(&user1, &1, &amount);

    let user = nft.read_user(&user1);
    let card: Card = nft.card(&user1, &TokenId(1)).unwrap();
    assert_eq!(user.power, 90);

    assert_eq!(card.clone().power, 1010);
}
// #[test]
// fn test_write_nft () {

//     let e = Env::default();
//     e.mock_all_auths();

//     let admin1 = Address::generate(&e);
//     let user: Address = Address::generate(&e);

//     let metadata = CardMetadata {
//         name: String::from_str(&e, "Tessa Trend"),
//         base_uri: String::from_str(&e, ""),
//         thumb_uri: String::from_str(&e, ""),
//         description: String::from_str(&e, ""),
//         initial_power: 1000, // Set appropriate value
//         max_power: 10000,     // Set appropriate value
//         level: 1,         // Set appropriate value
//         category: Category::Leader, // Example category
//         price_xtar: 100,    // Set appropriate value
//         price_terry: 100,   // Set appropriate value
//         token_id: 101,
//     };

//     let nft = create_nft(e.clone(), &admin1, &config);
//     nft.create_metadata(&metadata, &101);
//     // nft.mint(&user1,  &TokenId(1), &1, &Currency::Terry);
//     assert!(nft.exists(&user,  &TokenId(101)) == true);

//     let card = nft.card(&user1, &TokenId(1)).unwrap();

//     write_nft(&e, user, TokenId(101), card);
// }

#[test]
fn test_stake() {
    let e = Env::default();
    e.mock_all_auths();
    
    std::println!("hello test stake");

    let admin1 = Address::generate(&e);
    let user1 = Address::generate(&e);
    // Generate config
    let mut config = generate_config(&e);

    let terry_token = e.register_stellar_asset_contract(admin1.clone());
    config.terry_token = terry_token.clone();
    let terry_token_client = TokenClient::new(&e, &terry_token);

    let nft = create_nft(e.clone(), &admin1, &config);

    // Mint terry tokens to user1
    mint_token(&e, config.terry_token.clone(), user1.clone(), 100000);
    assert_eq!(terry_token_client.balance(&user1), 100000);

    // Set user level
    //nft.set_user_level(&user1.clone(), &1);

    let metadata = create_metadata(&e);
    nft.create_metadata(&metadata, &1);

    // Mint token 1 to user1
    assert!(nft.exists(&user1,  &TokenId(1)) == false);

    nft.mint(&user1, &TokenId(1), &1, &Currency::Terry);
    assert!(nft.exists(&user1,  &TokenId(1)) == true);

    let card_info = CardInfo::get_default_card(Category::Leader);

    // Stake
    let mut card = nft.card(&user1,  &TokenId(1)).unwrap();
    let power_action_fee = config.power_action_fee * card.power / 100;
    nft.stake(&user1, &Category::Leader, &TokenId(1), &0);
    card = nft.card(&user1,  &TokenId(1)).unwrap();
    
    assert!(card.locked_by_action == Action::Stake);
    assert_eq!(card.power, 0);
    assert_eq!(power_action_fee, 20);

    let balance = nft.admin_balance();
    assert!(balance.haw_ai_power == power_action_fee);

    let stake = nft.read_stake(&user1, &Category::Leader, &TokenId(1));
    assert_eq!(stake.power, 1000 - power_action_fee);

    // Unstake
    nft.unstake(&user1, &Category::Leader, &TokenId(1));

    // assert!(nft.config().terry_token == config.terry_token);
    // // nft.mint_token(&config.terry_token, &user1, &100);
    // mint_token(&e, config.terry_token, user1.clone(), 100);

    // assert!(e.ledger().sequence() == 0);
    // nft.stake(&user1, &Category::Leader, &TokenId(1), &0);
}

#[test]
fn test_fight() {
    let e = Env::default();
    e.mock_all_auths();

    let admin1 = Address::generate(&e);
    let user1 = Address::generate(&e);

    // Generate config
    let mut config = generate_config(&e);

    let terry_token = e.register_stellar_asset_contract(admin1.clone());
    config.terry_token = terry_token.clone();
    let terry_token_client = TokenClient::new(&e, &terry_token);

    let nft = create_nft(e.clone(), &admin1, &config);

    // Mint terry tokens to user1
    mint_token(&e, config.terry_token.clone(), user1.clone(), 1000);
    assert_eq!(terry_token_client.balance(&user1), 1000);

    // Set user level
    //nft.set_user_level(&user1.clone(), &1);

    let metadata = create_metadata(&e);
    nft.create_metadata(&metadata, &1);

    // Mint token 1 to user1
    assert!(nft.exists(&user1,  &TokenId(1)) == false);
    nft.mint(&user1,  &TokenId(1), &1, &Currency::Terry);
    assert!(nft.exists(&user1,  &TokenId(1)) == true);

    let card_info = CardInfo::get_default_card(Category::Leader);

    // Fight
    nft.open_position(
        &user1,
        &Category::Leader,
        &TokenId(1),
        &fight::FightCurrency::BTC,
        &SidePosition::Long,
        &120,
    );
    nft.close_position(&user1.clone(), &Category::Leader, &TokenId(1));
}

#[test]
fn test_lending() {
    let e = Env::default();
    e.mock_all_auths();

    let admin1 = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);

    // Generate config
    let mut config = generate_config(&e);

    let terry_token = e.register_stellar_asset_contract(admin1.clone());
    config.terry_token = terry_token.clone();
    let terry_token_client = TokenClient::new(&e, &terry_token);

    let nft = create_nft(e.clone(), &admin1, &config);

    let metadata = create_metadata(&e);
    nft.create_metadata(&metadata, &1);

    // Mint terry tokens to user1 & user2
    mint_token(&e, config.terry_token.clone(), user1.clone(), 1000);
    assert_eq!(terry_token_client.balance(&user1), 1000);
    mint_token(&e, config.terry_token.clone(), user2.clone(), 1000);
    assert_eq!(terry_token_client.balance(&user2), 1000);
    // Set user level
    //nft.set_user_level(&user1.clone(), &1);
    //nft.set_user_level(&user2.clone(), &1);

    // Mint token 1 to user1
    assert!(nft.exists(&user1, &TokenId(1)) == false);
    nft.mint(&user1, &TokenId(1), &1, &Currency::Terry);
    assert!(nft.exists(&user1, &TokenId(1)) == true);

    assert!(nft.exists(&user2, &TokenId(1)) == false);
    nft.mint(&user2, &TokenId(1), &1, &Currency::Terry);
    assert!(nft.exists(&user2, &TokenId(1)) == true);

    // Create a Lend token 1
    nft.lend(&user1, &Category::Resource, &TokenId(1), &100);
    nft.borrow(&user2, &Category::Resource, &TokenId(1), &70);
    nft.repay(&user2, &Category::Resource, &TokenId(1));
    nft.withdraw(&user1, &Category::Resource, &TokenId(1));
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

    let terry_token = e.register_stellar_asset_contract(admin1.clone());
    config.terry_token = terry_token.clone();
    let terry_token_client = TokenClient::new(&e, &terry_token);

    let nft = create_nft(e.clone(), &admin1, &config);

    // Mint terry tokens to user1 & user2
    mint_token(&e, config.terry_token.clone(), user1.clone(), 1000);
    assert_eq!(terry_token_client.balance(&user1), 1000);
    mint_token(&e, config.terry_token.clone(), user2.clone(), 1000);
    assert_eq!(terry_token_client.balance(&user2), 1000);
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

// #[test]
// #[should_panic(expected = "already initialized")]
// fn test_initialize_already_initialized() {
//     let e = Env::default();
//     let admin = Address::generate(&e);

//     // Generate config
//     let config = generate_config(&e);

//     let nft = create_nft(e, &admin, &config);

//     // Try to initialize again
//     nft.initialize(&admin, &config);
// }

// #[test]
// fn test_set_admin() {
//     let e = Env::default();
//     e.mock_all_auths();

//     let admin1 = Address::generate(&e);
//     let admin2 = Address::generate(&e);

//     // Generate config
//     let config = generate_config(&e);

//     let nft = create_nft(e, &admin1, &config);

//     // Set new admin
//     nft.set_admin(&admin2);
// }


// #[test]
// fn test_burn() {
//     let e = Env::default();
//     e.mock_all_auths();

//     let admin1 = Address::generate(&e);
//     let user1 = Address::generate(&e);
//     let user2 = Address::generate(&e);

//     // Generate config
//     let mut config = generate_config(&e);

//     let terry_token = e.register_stellar_asset_contract(admin1.clone());
//     config.terry_token = terry_token.clone();
//     let terry_token_client = TokenClient::new(&e, &terry_token);

//     let nft = create_nft(e.clone(), &admin1, &config);
//     nft.set_admin(&admin1);

//     let card: CardMetadata =  CardMetadata {
//         name : String::from_str(&e, "Jed") ,
//         base_uri: String::from_str(&e, "asdasdasd") ,
//         thumb_uri: String::from_str(&e, "asdasdasd") ,
//         description: String::from_str(&e, "asdasdasd") ,
//         initial_power: 100,
//         max_power: 1000,
//         level: 1,
//         category: Category::Leader,
//         price_xtar: 10000,
//         price_terry: 400,
//         token_id: 1,
//     };

//     nft.create_metadata( &card, &1);
//     // Mint terry tokens to user1
//     mint_token(&e, config.terry_token.clone(), user1.clone(), 100000);
//     assert_eq!(terry_token_client.balance(&user1), 100000);

//     // Set user level
//     //nft.set_user_level(&user1.clone(), &1);

//     // Mint token 1 to user1
//     assert!(nft.exists(&user2,  &TokenId(1)) == false);
//     nft.create_user( &user1,&user2);
//     nft.mint(&user1,  &TokenId(1), &1, &Currency::Terry);


//     assert!(nft.exists(&user2, &TokenId(1)) == true);

//     // Burn
//     // Mint terry tokens to admin
//     mint_token(&e, config.terry_token.clone(), user2.clone(), 100000);
//    // assert_eq!(terry_token_client.balance(&admin1), 100000);
//     nft.transfer_terry_contract(&user2, &100000);

//    nft.burn(&user1, &TokenId(1));

//    assert!(nft.exists(&user1,  &TokenId(1)) == false);


// }

// #[test]
// fn test_currency_price() {
//     let e = Env::default();
//     e.mock_all_auths();

//     let admin1 = Address::generate(&e);
//     // let user1 = Address::generate(&e);

//     // Generate config
//     let mut config = generate_config(&e);

//     config.oracle_contract_id = Address::from_string(&String::from_str(&e, "CBKZFI26PDCZUJ5HYYKVB5BWCNYUSNA5LVL4R2JTRVSOB4XEP7Y34OPN"));

//     let nft = create_nft(e, &admin1, &config);

//     let price = nft.currency_price(&config.oracle_contract_id );//&fight::FightCurrency::BTC);
//     assert!(price == 0);
// }

#[test]
fn test_transfer_terry() {
    let e = Env::default();
    e.mock_all_auths();
    log!(&e, "test nft mint function");

    let admin1 = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);

    let mut config = generate_config(&e);

    let terry_token = e.register_stellar_asset_contract(admin1.clone());
    config.terry_token = terry_token.clone();
    let xtar_token = e.register_stellar_asset_contract(admin1.clone());
    config.xtar_token = xtar_token.clone();
    let terry_token_client = TokenClient::new(&e, &terry_token);
    let xtar_token_client = TokenClient::new(&e, &xtar_token);
    
    // Mint terry tokens to user1
    mint_token(&e, config.terry_token.clone(), user1.clone(), 100000);
    assert_eq!(terry_token_client.balance(&user1), 100000);

    // Mint xtar tokens to user2
    mint_token(&e, config.xtar_token.clone(), user2.clone(), 100000);
    assert_eq!(xtar_token_client.balance(&user2), 100000);

    transfer_terry(&e, user1.clone(), 1000);

    assert_eq!(terry_token_client.balance(&user1), 101000);
}
