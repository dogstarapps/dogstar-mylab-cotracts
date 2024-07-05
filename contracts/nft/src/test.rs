#![cfg(test)]
extern crate std;

use crate::{contract::NFT, storage_types::TokenId, NFTClient};
use soroban_sdk::{
    testutils::Address as _, vec, Address, Env, IntoVal
};

fn create_nft<'a>(e: &Env, admin: &Address) -> NFTClient<'a> {
    let nft: NFTClient = NFTClient::new(e, &e.register_contract(None, NFT {}));
    nft.initialize(
        admin,
        &"name".into_val(e),
        &"symbol".into_val(e),
        &"base_uri".into_val(e),
    );
    nft
}

#[test]
fn test() {
    let e = Env::default();
    e.mock_all_auths();

    let admin1 = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);
    let user3 = Address::generate(&e);
    let nft = create_nft(&e, &admin1);
    nft.set_user_level(&user1.clone(), &1);
    // Mint token 1 to user1
    nft.mint(&user1, &TokenId(1), &1);
    assert_eq!(nft.balance_of(&user1.clone()), 1);
    assert_eq!(nft.owner_of(&TokenId(1)), user1.clone());

    // Approve transfer of token 1 from user1 to user2
    nft.approve(&user2.clone(), &TokenId(1));
    assert_eq!(nft.get_approved(&TokenId(1)), Some(user2.clone()));

    // Transfer token 1 from user1 to user2
    nft.transfer_from(&user1, &user2, &TokenId(1));
    assert_eq!(nft.owner_of(&TokenId(1)), user2.clone());
    assert_eq!(nft.get_approved(&TokenId(1)), None);

    // Set approval for all from user2 to user3
    nft.set_approval_for_all(&user2.clone(), &user3.clone(), &true);
    assert!(nft.is_approved_for_all(&user2.clone(), &user3.clone()));
    nft.transfer_from(&user3.clone(), &user1.clone(), &TokenId(1));
    assert_eq!(nft.owner_of(&TokenId(1)), user1.clone());

    // Mint token 2 to user1
    nft.mint(&user1, &TokenId(2), &1);
    nft.approve(&user2.clone(), &TokenId(2));
    // Transfer token 2 from user1 to user3 using operator user2
    nft.transfer_from(&user2.clone(), &user3.clone(), &TokenId(2));
    assert_eq!(nft.owner_of(&TokenId(2)), user3.clone());

    // Burn token 2 from user3
    nft.burn(&user3.clone(), &TokenId(2));
    assert_eq!(nft.balance_of(&user3.clone()), 0);
    assert!(!nft.exists(&TokenId(2)));
}

#[test]
#[should_panic(expected = "already initialized")]
fn initialize_already_initialized() {
    let e = Env::default();
    let admin = Address::generate(&e);
    let nft = create_nft(&e, &admin);

    nft.initialize(
        &admin,
        &"name".into_val(&e),
        &"symbol".into_val(&e),
        &"base_uri".into_val(&e),
    );
}

#[test]
#[should_panic(expected = "Caller is not owner nor approved")]
fn test_transfer_from_not_owner() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);
    let nft = create_nft(&e, &admin);

    nft.set_user_level(&user1.clone(), &1);
    // Mint token to user1
    nft.mint(&user1.clone(), &TokenId(1), &1);
    assert_eq!(nft.balance_of(&user1.clone()), 1);

    // Try to transfer token 1 from user1 to user2 (should fail since caller is not owner)
    nft.transfer_from(&user2.clone(), &user1.clone(), &TokenId(1));
}

#[test]
#[should_panic(expected = "Caller is not owner nor approved")]
fn test_transfer_from_insufficient_allowance() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);
    let user3 = Address::generate(&e);
    let nft = create_nft(&e, &admin);

    nft.set_user_level(&user1.clone(), &1);

    // Mint token to user1
    nft.mint(&user1.clone(), &TokenId(1), &1);
    assert_eq!(nft.balance_of(&user1.clone()), 1);

    // Approve transfer of token 1 from user1 to user2
    nft.approve(&user2.clone(), &TokenId(1));
    assert_eq!(
        nft.get_approved(&TokenId(1)),
        Some(user2.clone())
    );

    // Try to transfer token 1 from user2 to user3 (should fail due to insufficient allowance)
    nft.transfer_from(&user3.clone(), &user1.clone(), &TokenId(1));
}

#[test]
#[should_panic(expected = "Token ID already exists")]
fn test_mint_existing_token() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let user1 = Address::generate(&e);
    let nft = create_nft(&e, &admin);

    nft.set_user_level(&user1.clone(), &1);

    // Mint token to user1
    nft.mint(&user1.clone(), &TokenId(1), &1);
    assert_eq!(nft.balance_of(&user1.clone()), 1);

    // Try to mint token 1 again (should fail since token ID already exists)
    nft.mint(&user1.clone(), &TokenId(1), &1);
}

#[test]
#[should_panic(expected = "User level too low to mint this card")]
fn test_mint_low_user_level() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let user1 = Address::generate(&e);
    let nft = create_nft(&e, &admin);

    // Mint token to user1
    nft.mint(&user1.clone(), &TokenId(1), &1);
    assert_eq!(nft.balance_of(&user1.clone()), 1);
}


#[test]
fn test_nft_lock_and_level() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let user1 = Address::generate(&e);
    let whitlist1 = Address::generate(&e);
    let nft = create_nft(&e, &admin);

    nft.add_to_whitelist(&vec![&e.clone(), whitlist1.clone()]);

    // Mint token to user1
    nft.mint(&user1.clone(), &TokenId(1), &0);
    assert_eq!(nft.balance_of(&user1.clone()), 1);

    nft.lock_nft(&whitlist1, &TokenId(1));
    nft.unlock_nft(&whitlist1, &TokenId(1));

    nft.upgrade_nft_level(&whitlist1, &TokenId(1), &1);
    assert_eq!(nft.nft_level(&TokenId(1)), 1);
}