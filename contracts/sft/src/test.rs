#![cfg(test)]
extern crate std;

use crate::{contract::SFT, storage_types::TokenId, SFTClient};
use soroban_sdk::{testutils::Address as _, vec, Address, Env, IntoVal, Vec};

fn create_sft<'a>(e: &Env, admin: &Address) -> SFTClient<'a> {
    let sft: SFTClient = SFTClient::new(e, &e.register_contract(None, SFT {}));
    sft.initialize(
        admin,
        &"name".into_val(e),
        &"symbol".into_val(e),
        &"base_uri".into_val(e),
    );
    sft
}

#[test]
fn test_mint() {
    let e = Env::default();
    e.mock_all_auths();

    let admin1 = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);
    let user3 = Address::generate(&e);
    let sft = create_sft(&e, &admin1);

    // Mint token 1 to user1
    sft.mint(&user1, &TokenId(1), &10);
    assert_eq!(sft.balance_of(&user1.clone(), &TokenId(1)), 10);
}

#[test]
fn test_batch_mint() {
    let e = Env::default();
    e.mock_all_auths();

    let admin1 = Address::generate(&e);
    let user1 = Address::generate(&e);
    let sft = create_sft(&e, &admin1);

    let ids: Vec<TokenId> = vec![&e.clone(), TokenId(1), TokenId(2)];
    let amounts: Vec<u64> = vec![&e.clone(), 10, 20];

    // Batch mint tokens to user1
    sft.mint_batch(&user1, &ids, &amounts);
    assert_eq!(sft.balance_of(&user1, &TokenId(1)), 10);
    assert_eq!(sft.balance_of(&user1, &TokenId(2)), 20);
}

#[test]
fn test_transfer() {
    let e = Env::default();
    e.mock_all_auths();

    let admin1 = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);
    let sft = create_sft(&e, &admin1);

    sft.mint(&user1, &TokenId(1), &10);

    // Transfer token 1 from user1 to user2
    sft.transfer(&user1, &user2, &TokenId(1), &5);
    assert_eq!(sft.balance_of(&user1, &TokenId(1)), 5);
    assert_eq!(sft.balance_of(&user2, &TokenId(1)), 5);
}

#[test]
fn test_batch_transfer() {
    let e = Env::default();
    e.mock_all_auths();

    let admin1 = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);
    let sft = create_sft(&e, &admin1);

    let ids: Vec<TokenId> = vec![&e.clone(), TokenId(1), TokenId(2)];
    let amounts: Vec<u64> = vec![&e.clone(), 10, 20];

    sft.mint_batch(&user1, &ids, &amounts);

    let transfer_amounts: Vec<u64> = vec![&e.clone(), 5, 10];
    sft.batch_transfer(&user1, &user2, &ids, &transfer_amounts);

    assert_eq!(sft.balance_of(&user1, &TokenId(1)), 5);
    assert_eq!(sft.balance_of(&user1, &TokenId(2)), 10);
    assert_eq!(sft.balance_of(&user2, &TokenId(1)), 5);
    assert_eq!(sft.balance_of(&user2, &TokenId(2)), 10);
}

#[test]
fn test_burn() {
    let e = Env::default();
    e.mock_all_auths();

    let admin1 = Address::generate(&e);
    let user1 = Address::generate(&e);
    let sft = create_sft(&e, &admin1);

    sft.mint(&user1, &TokenId(1), &10);

    // Burn token 1 from user1
    sft.burn(&user1, &TokenId(1), &5);
    assert_eq!(sft.balance_of(&user1, &TokenId(1)), 5);
}

#[test]
fn test_batch_burn() {
    let e = Env::default();
    e.mock_all_auths();

    let admin1 = Address::generate(&e);
    let user1 = Address::generate(&e);
    let sft = create_sft(&e, &admin1);

    let ids: Vec<TokenId> = vec![&e.clone(), TokenId(1), TokenId(2)];
    let amounts: Vec<u64> = vec![&e.clone(), 10, 20];

    sft.mint_batch(&user1, &ids, &amounts);

    let burn_amounts: Vec<u64> = vec![&e.clone(), 5, 10];
    sft.burn_batch(&user1, &ids, &burn_amounts);

    assert_eq!(sft.balance_of(&user1, &TokenId(1)), 5);
    assert_eq!(sft.balance_of(&user1, &TokenId(2)), 10);
}

#[test]
fn test_balance_of_batch() {
    let e = Env::default();
    e.mock_all_auths();

    let admin1 = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);
    let sft = create_sft(&e, &admin1);

    let ids: Vec<TokenId> = vec![&e.clone(), TokenId(1), TokenId(2)];
    let amounts: Vec<u64> = vec![&e.clone(), 10, 20];

    sft.mint_batch(&user1, &ids, &amounts);

    let owners: Vec<Address> = vec![&e.clone(), user1.clone(), user2.clone()];
    let balances = sft.balance_of_batch(&owners, &ids);
    assert_eq!(balances.get(0).unwrap(), 10);
    assert_eq!(balances.get(1).unwrap(), 0);
}

#[test]
#[should_panic(expected = "already initialized")]
fn initialize_already_initialized() {
    let e = Env::default();
    let admin = Address::generate(&e);
    let sft = create_sft(&e, &admin);

    sft.initialize(
        &admin,
        &"name".into_val(&e),
        &"symbol".into_val(&e),
        &"base_uri".into_val(&e),
    );
}
