//! This contract demonstrates a sample implementation of the Soroban token
//! interface.

use crate::admin::{
    self, has_administrator, is_whitelisted, read_administrator, read_config, write_administrator, write_config, Config
};
use crate::nft_info::{
    exists, read_nft, remove_nft, write_nft, Action, CardInfo, Category, Currency,
};
use crate::storage_types::{DataKey, TokenId, INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD};
use crate::user_info::read_user_level;
use soroban_sdk::{contract, contractimpl, token, Address, BytesN, Env, Vec};
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

    pub fn nft_level(env: Env, owner: Address, category: Category, token_id: TokenId) -> u32 {
        read_nft(&env, owner, category, token_id).dl_level
    }

    pub fn user_level(env: Env, user: Address) -> u32 {
        read_user_level(&env, user)
    }

    pub fn nft_locked(env: Env, owner: Address, category: Category, token_id: TokenId) -> Action {
        read_nft(&env, owner, category, token_id).locked_by_action
    }

    pub fn mint(
        env: Env,
        to: Address,
        category: Category,
        token_id: TokenId,
        card_level: u32,
        buy_currency: Currency,
    ) {
        let admin = read_administrator(&env);
        let user_level = read_user_level(&env, to.clone());
        assert!(
            user_level >= card_level,
            "User level too low to mint this card"
        );
        assert!(
            !Self::exists(&env, to.clone(), category.clone(), token_id.clone()),
            "Token ID already exists"
        );
        let mut nft = CardInfo::get_default_card(category.clone());
        nft.dl_level = card_level;
        write_nft(&env, to.clone(), category, token_id, nft.clone());

        // puchase by currency
        let config = read_config(&env);
        if buy_currency == Currency::Terry {
            let token = token::Client::new(&env, &config.terry_token.clone());
            let withdrawable_amount = (config.withdrawable_percentage as i128) * nft.price_terry / 100;
            let haw_ai_amount = nft.price_terry - withdrawable_amount;
            token.transfer(&to.clone(), &admin, &withdrawable_amount);
            token.transfer(&to.clone(), &config.haw_ai_pot, &haw_ai_amount);

        } else {
            let token = token::Client::new(&env, &config.xtar_token.clone());
            let burnable_amount = (config.burnable_percentage as i128) * nft.price_xtar / 100;
            let haw_ai_amount = nft.price_terry - burnable_amount;
            token.burn(&to.clone(), &burnable_amount);
            token.transfer(&to.clone(), &config.haw_ai_pot, &haw_ai_amount);
        };

    }

    pub fn transfer(env: Env, from: Address, to: Address, category: Category, token_id: TokenId) {
        let nft = read_nft(&env, from.clone(), category.clone(), token_id.clone());
        remove_nft(&env, from.clone(), category.clone(), token_id.clone());
        write_nft(&env, to, category, token_id, nft);
    }

    pub fn burn(env: Env, owner: Address, category: Category, token_id: TokenId) {
        remove_nft(&env, owner, category, token_id);
    }

    pub fn exists(env: &Env, owner: Address, category: Category, token_id: TokenId) -> bool {
        exists(env, owner, category, token_id)
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

    // pub fn upgrade_nft_level(e: Env, caller: Address, token_id: TokenId, new_level: u64) {
    //     caller.require_auth();
    //     let admin = read_administrator(&e);
    //     let whitelisted = is_whitelisted(&e, &caller.clone());

    //     assert!(caller == admin || whitelisted, "Caller is not admin or whitelisted");

    //     write_nft_level(&e, token_id, new_level);
    // }

    // pub fn lock_nft(e: Env, caller: Address, token_id: TokenId) {
    //     caller.require_auth();
    //     let admin = read_administrator(&e);
    //     let whitelisted = is_whitelisted(&e, &caller.clone());

    //     assert!(caller == admin || whitelisted, "Caller is not admin or whitelisted");

    //     write_nft_lock(&e, token_id, Some(caller));
    // }

    // pub fn unlock_nft(e: Env, caller: Address, token_id: TokenId) {
    //     caller.require_auth();
    //     let locker = read_nft_lock(&e, token_id.clone());
    //     assert_ne!(locker, None, "Can't unlock");
    //     assert_eq!(caller, locker.unwrap(), "Caller did not lock this NFT");

    //     write_nft_lock(&e, token_id, None);
    // }

    pub fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        let admin: Address = e.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }
}
