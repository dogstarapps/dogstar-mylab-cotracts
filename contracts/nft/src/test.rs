#![cfg(test)]
extern crate std;

use crate::nft_info::Card;
use crate::NFTClient;
use crate::{
    admin::Config,
    contract::NFT,
    metadata::CardMetadata,
    nft_info::{Category, Currency},
    storage_types::TokenId,
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
    nft.create_user(&player, &player);

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
