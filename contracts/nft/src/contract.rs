//! This contract demonstrates a sample implementation of the Soroban token
//! interface.

use crate::admin::{has_administrator, read_administrator, write_administrator, is_whitelisted};
use crate::allowance::{read_approved, write_approved, read_approval_for_all, write_approval_for_all};
use crate::balance::{read_balance, write_balance, read_owner, write_owner};
use crate::metadata::{read_metadata, write_metadata, NFTMetadata};
use crate::nft_info::{read_nft_level, read_nft_lock, write_nft_level, write_nft_lock};
use crate::storage_types::{DataKey, TokenId};
use crate::storage_types::{INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT};
use crate::user_info::read_user_level;
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String, Vec};
use soroban_token_sdk::TokenUtils;

#[contract]
pub struct NFT;

#[contractimpl]
impl NFT {
    pub fn initialize(
        e: Env,
        admin: Address,
        name: String,
        symbol: String,
        base_uri: String,
    ) {
        if has_administrator(&e) {
            panic!("already initialized")
        }
        write_administrator(&e, &admin);

        write_metadata(
            &e,
            NFTMetadata {
                name,
                symbol,
                base_uri,
            },
        )
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

    pub fn name(env: Env) -> String {
        read_metadata(&env).name
    }

    pub fn symbol(env: Env) -> String {
        read_metadata(&env).symbol
    }

    pub fn base_uri(env: Env) -> String {
        read_metadata(&env).base_uri
    }

    pub fn nft_level(env: Env, token_id: TokenId) -> u64 {
        read_nft_level(&env, token_id)
    }

    pub fn user_level(env: Env, user: Address) -> u64 {
        read_user_level(&env, user)
    }

    pub fn nft_locked(env: Env, token_id: TokenId) -> Option<Address> {
        read_nft_lock(&env, token_id)
    }

    pub fn mint(env: Env, to: Address, token_id: TokenId, card_level: u64) {
        let user_level = read_user_level(&env, to.clone());
        assert!(user_level >= card_level, "User level too low to mint this card");
        assert!(!Self::exists(&env, token_id.clone()), "Token ID already exists");
        write_owner(&env, token_id.clone(), Some(to.clone()));
        write_balance(&env, to.clone(), read_balance(&env, to) + 1);
        write_nft_level(&env, token_id.clone(), card_level);
    }
    
    pub fn balance_of(env: Env, owner: Address) -> u64 {
        read_balance(&env, owner)
    }

    pub fn owner_of(env: Env, token_id: TokenId) -> Address {
        read_owner(&env, token_id)
    }

    pub fn approve(env: Env, approved: Address, token_id: TokenId) {
        let owner = read_owner(&env, token_id.clone());
        owner.require_auth();
        
        write_approved(&env, token_id, Some(approved));
    }

    pub fn get_approved(env: Env, token_id: TokenId) -> Option<Address> {
        read_approved(&env, token_id)
    }

    pub fn set_approval_for_all(env: Env, owner: Address, operator: Address, approved: bool) {
        owner.require_auth();

        write_approval_for_all(&env, owner, operator, approved);
    }

    pub fn is_approved_for_all(env: Env, owner: Address, operator: Address) -> bool {
        read_approval_for_all(&env, owner, operator)
    }

    pub fn transfer_from(env: Env, spender: Address, to: Address, token_id: TokenId) {
        let owner = read_owner(&env, token_id.clone());
        spender.require_auth();
        assert!(
            spender == owner
                || Some(spender.clone()) == read_approved(&env, token_id.clone())
                || read_approval_for_all(&env, owner.clone(), spender.clone()),
            "Caller is not owner nor approved"
        );
        write_owner(&env, token_id.clone(), Some(to.clone()));
        write_balance(&env, owner.clone(), read_balance(&env, owner) - 1);
        write_balance(&env, to.clone(), read_balance(&env, to) + 1);
        write_approved(&env, token_id.clone(), None);
    }

    pub fn burn(env: Env, from: Address, token_id: TokenId) {
        let owner = read_owner(&env, token_id.clone());
        assert_eq!(from, owner, "Caller is not the owner");
        write_owner(&env, token_id.clone(), None);
        write_balance(&env, owner.clone(), read_balance(&env, owner) - 1);
    }

    pub fn exists(env: &Env, token_id: TokenId) -> bool {
        let key = DataKey::Owner(token_id);
        env.storage().persistent().has(&key)
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

    pub fn set_user_level(e: Env, user: Address, level: u64) {
        let admin = read_administrator(&e);
        admin.require_auth();

        e.storage()
            .persistent()
            .set(&DataKey::UserLevel(user.clone()), &level);
    }

    pub fn upgrade_nft_level(e: Env, caller: Address, token_id: TokenId, new_level: u64) {
        caller.require_auth();
        let admin = read_administrator(&e);
        let whitelisted = is_whitelisted(&e, &caller.clone());

        assert!(caller == admin || whitelisted, "Caller is not admin or whitelisted");

        write_nft_level(&e, token_id, new_level);
    }

    pub fn lock_nft(e: Env, caller: Address, token_id: TokenId) {
        caller.require_auth();
        let admin = read_administrator(&e);
        let whitelisted = is_whitelisted(&e, &caller.clone());

        assert!(caller == admin || whitelisted, "Caller is not admin or whitelisted");

        write_nft_lock(&e, token_id, Some(caller));
    }

    pub fn unlock_nft(e: Env, caller: Address, token_id: TokenId) {
        caller.require_auth();
        let locker = read_nft_lock(&e, token_id.clone());
        assert_ne!(locker, None, "Can't unlock");
        assert_eq!(caller, locker.unwrap(), "Caller did not lock this NFT");

        write_nft_lock(&e, token_id, None);
    }

    pub fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        let admin: Address = e.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }
}
