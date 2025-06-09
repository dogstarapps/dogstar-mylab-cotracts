use crate::contract::NFT;
use crate::{user_info::mint_terry, *};
use admin::read_config;
use nft_info::{read_nft, remove_nft};
use soroban_sdk::{Address, Env};
use storage_types::TokenId;
use user_info::{read_owner_card, read_user, write_owner_card};

pub fn burn(env: Env, user: Address, token_id: TokenId) {
    //user.require_auth();
    let owner = read_user(&env, user.clone()).owner;

    let config = read_config(&env);
    let nft = read_nft(&env, owner.clone(), token_id.clone()).unwrap();

    // Calculate Terry and Power amounts
    let terry_amount = config.terry_per_power * nft.power as i128;
    let receive_amount = terry_amount * config.burn_receive_percentage as i128 / 100;
    let pot_terry = terry_amount - receive_amount; // Terry to pot
    let pot_power = nft.power * (100 - config.burn_receive_percentage) / 100; // Power to pot

    // Mint owner's share
    mint_terry(&env, user.clone(), receive_amount);

    // Accumulate to pot with Dogstar fee deduction
    NFT::accumulate_pot(env.clone(), pot_terry, pot_power, 0);

    // Remove card and NFT
    remove_owner_card(&env, owner.clone(), token_id.clone());
    remove_nft(&env, owner, token_id);
}

pub fn remove_owner_card(env: &Env, owner: Address, token_id: TokenId) {
    let mut user_card_ids = read_owner_card(&env, owner.clone());
    let index = user_card_ids.iter().position(|x| x == token_id).unwrap();
    user_card_ids.remove(index as u32);
    write_owner_card(&env, owner.clone(), user_card_ids);
}
