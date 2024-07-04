//! This contract demonstrates a sample implementation of the Soroban token
//! interface.

use crate::admin::{has_administrator, read_administrator, write_administrator};
use crate::allowance::{read_approval_for_all, read_approved, write_approval_for_all};
use crate::balance::{read_balance, write_balance};
use crate::metadata::{read_metadata, write_metadata, SFTMetadata};
use crate::storage_types::{TokenId};
use crate::storage_types::{INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT};
use soroban_sdk::{contract, contractimpl, vec, Address, Env, String, Vec};
use soroban_token_sdk::TokenUtils;

#[contract]
pub struct SFT;

#[contractimpl]
impl SFT {
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
            SFTMetadata {
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

    pub fn mint(
        env: Env,
        to: Address,
        id: TokenId,
        amount: u64,
    ) {
        assert!(amount > 0, "Amount must be greater than zero");
        write_balance(&env, to.clone(), id.clone(), read_balance(&env, to.clone(), id.clone()) + amount);
    }

    pub fn mint_batch(
        env: Env,
        to: Address,
        ids: Vec<TokenId>,
        amounts: Vec<u64>,
    ) {
        assert!(ids.len() == amounts.len(), "IDs and amounts length mismatch");
        for i in 0..ids.len() {
            let id = ids.get(i).unwrap();
            let amount = amounts.get(i).unwrap();
            assert!(amount > 0, "Amount must be greater than zero");
            write_balance(&env, to.clone(), id.clone(), read_balance(&env, to.clone(), id.clone()) + amount);
        }
    }

    pub fn balance_of(env: Env, owner: Address, id: TokenId) -> u64 {
        read_balance(&env, owner, id)
    }

    pub fn balance_of_batch(env: Env, owners: Vec<Address>, ids: Vec<TokenId>) -> Vec<u64> {
        assert!(owners.len() == ids.len(), "Owners and IDs length mismatch");
        let mut balances = vec![&env.clone()];
        for i in 0..owners.len() {
            let owner = owners.get(i).unwrap();
            let id = ids.get(i).unwrap();
            balances.push_back(read_balance(&env, owner.clone(), id.clone()));
        }
        balances
    }

    pub fn set_approval_for_all(env: Env, owner: Address, operator: Address, approved: bool) {
        owner.require_auth();
        write_approval_for_all(&env, owner.clone(), operator, approved);
    }

    pub fn is_approved_for_all(env: Env, owner: Address, operator: Address) -> bool {
        read_approval_for_all(&env, owner, operator)
    }

    pub fn transfer(
        env: Env,
        from: Address,
        to: Address,
        id: TokenId,
        amount: u64,
    ) {
        assert!(amount > 0, "Amount must be greater than zero");
        from.require_auth();
        
        write_balance(&env, from.clone(), id.clone(), read_balance(&env, from.clone(), id.clone()) - amount);
        write_balance(&env, to.clone(), id.clone(), read_balance(&env, to.clone(), id.clone()) + amount);
    }

    pub fn transfer_from(
        env: Env,
        spender: Address,
        from: Address,
        to: Address,
        id: TokenId,
        amount: u64,
    ) {
        assert!(amount > 0, "Amount must be greater than zero");
        spender.require_auth();
        
        assert!(
            spender == from || read_approval_for_all(&env, from.clone(), spender.clone()) || Some(spender.clone()) == read_approved(&env, id.clone()),
            "Caller is not owner nor approved"
        );
        write_balance(&env, from.clone(), id.clone(), read_balance(&env, from.clone(), id.clone()) - amount);
        write_balance(&env, to.clone(), id.clone(), read_balance(&env, to.clone(), id.clone()) + amount);
    }

    pub fn batch_transfer(
        env: Env,
        from: Address,
        to: Address,
        ids: Vec<TokenId>,
        amounts: Vec<u64>,
    ) {
        assert!(ids.len() == amounts.len(), "IDs and amounts length mismatch");
        from.require_auth();
        
        for i in 0..ids.len() {
            let id = ids.get(i).unwrap();
            let amount = amounts.get(i).unwrap();
            assert!(amount > 0, "Amount must be greater than zero");
            write_balance(&env, from.clone(), id.clone(), read_balance(&env, from.clone(), id.clone()) - amount);
            write_balance(&env, to.clone(), id.clone(), read_balance(&env, to.clone(), id.clone()) + amount);
        }
    }

    pub fn burn(
        env: Env,
        from: Address,
        id: TokenId,
        amount: u64,
    ) {
        assert!(amount > 0, "Amount must be greater than zero");
        from.require_auth();
        let balance = read_balance(&env, from.clone(), id.clone());
        assert!(balance >= amount, "Burn amount exceeds balance");
        write_balance(&env, from.clone(), id.clone(), balance - amount);
    }

    pub fn burn_batch(
        env: Env,
        from: Address,
        ids: Vec<TokenId>,
        amounts: Vec<u64>,
    ) {
        assert!(ids.len() == amounts.len(), "IDs and amounts length mismatch");
        from.require_auth();
        for i in 0..ids.len() {
            let id = ids.get(i).unwrap();
            let amount = amounts.get(i).unwrap();
            assert!(amount > 0, "Amount must be greater than zero");
            let balance = read_balance(&env, from.clone(), id.clone());
            assert!(balance >= amount, "Burn amount exceeds balance");
            write_balance(&env, from.clone(), id.clone(), balance - amount);
        }
    }
}
