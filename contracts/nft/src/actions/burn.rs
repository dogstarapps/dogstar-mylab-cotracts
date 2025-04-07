use crate::*;
use admin::{transfer_terry, read_balance, read_config, write_balance};
use nft_info::{read_nft, remove_nft, Category};
use soroban_sdk::{Address, Env};
use storage_types::TokenId;
use user_info::{ read_user, read_owner_card, write_owner_card };

pub fn burn(env: Env, fee_payer: Address,  token_id: TokenId) {
    // fee_payer.require_auth();
    let owner = read_user(&env, fee_payer.clone()).owner;

    let config = read_config(&env);

    let nft = read_nft(&env, owner.clone(), token_id.clone()).unwrap();

    let terry_amount = config.terry_per_power * nft.power as i128;
    let receive_amount = terry_amount * config.burn_receive_percentage as i128 / 100;
    let haw_ai_amount = terry_amount - receive_amount;
    
    // mint 50% of terry amount to the owner
    // transfer_terry(&env, fee_payer.clone(), receive_amount);

    
    // mint 50% of terry amount to the haw ai pot
    // transfer_terry(&env, config.haw_ai_pot.clone(), haw_ai_amount);

    let mut balance = read_balance(&env);
    // update haw ai terry balance and power balance
    balance.haw_ai_terry += haw_ai_amount;
    let haw_ai_power = nft.power * config.burn_receive_percentage / 100;
    balance.haw_ai_power += haw_ai_power;
    write_balance(&env, &balance);

    remove_owner_card(&env, owner.clone(), token_id.clone());
    remove_nft(&env, owner, token_id);
       
}

pub fn remove_owner_card(env: &Env, owner: Address, token_id: TokenId) {
    let mut user_card_ids = read_owner_card(&env, owner.clone());
    let index = user_card_ids.iter().position(|x| x == token_id).unwrap();
    user_card_ids.remove(index as u32);
    write_owner_card(&env, owner.clone(), user_card_ids);
}

