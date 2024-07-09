use crate::*;
use contract::NFT;
use nft_info::{read_nft, write_nft, Action, Category};
use soroban_sdk::{contracttype, Address, Env, Vec};
use storage_types::{DataKey, TokenId, BALANCE_BUMP_AMOUNT, BALANCE_LIFETIME_THRESHOLD};

#[contracttype]
#[derive(Clone, PartialEq)]
pub struct LendingInfo {
    pub lender: Address,
    pub category: Category,
    pub token_id: TokenId,
    pub power: u32,
    pub interest_rate: u32,
    pub duration: u32,
    pub is_borrowed: bool,
    pub borrower: Address,
    pub collateral_category: Category,
    pub collateral_token_id: TokenId,
    pub borrowed_block: u32,
}

pub fn write_lending_info(
    env: Env,
    owner: Address,
    category: Category,
    token_id: TokenId,
    lending_info: LendingInfo,
) {
    let key = DataKey::Lending(owner.clone(), category.clone(), token_id.clone());
    env.storage().persistent().set(&key, &lending_info);
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

pub fn read_lending_info(env: Env, owner: Address, category: Category, token_id: TokenId) -> LendingInfo {
    let key = DataKey::Lending(owner.clone(), category.clone(), token_id.clone());
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
    env.storage().persistent().get(&key).unwrap()
}

pub fn remove_lending_info(env: Env, owner: Address, category: Category, token_id: TokenId) {
    let key = DataKey::Lending(owner.clone(), category.clone(), token_id.clone());
    env.storage().persistent().remove(&key);
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

pub fn add_lending(env: Env, lending_info: LendingInfo) {
    let key = DataKey::Lendings;
    let mut lendings = read_lendings(env.clone());
    lendings.push_back(lending_info);
    env.storage().persistent().set(&key, &lendings);
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

pub fn remove_lending(env: Env, lender: Address, category: Category, token_id: TokenId) {
    let key = DataKey::Lendings;
    let mut lendings = read_lendings(env.clone());
    if let Some(pos) = lendings.iter().position(|lending| {
        lending.lender == lender && lending.category == category && lending.token_id == token_id
    }) {
        lendings.remove(pos.try_into().unwrap());
    }

    env.storage().persistent().set(&key, &lendings);

    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

pub fn update_lending(env: Env, lender: Address, category: Category, token_id: TokenId, lending_info: LendingInfo) {
    let key = DataKey::Lendings;
    let mut lendings = read_lendings(env.clone());
    if let Some(pos) = lendings.iter().position(|lending| {
        lending.lender == lender && lending.category == category && lending.token_id == token_id
    }) {
        lendings.set(pos.try_into().unwrap(), lending_info)
    }

    env.storage().persistent().set(&key, &lendings);

    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

pub fn read_lendings(env: Env) -> Vec<LendingInfo> {
    let key = DataKey::Lendings;
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
    env.storage().persistent().get(&key).unwrap()
}

impl NFT {
    pub fn lend(
        env: Env,
        owner: Address,
        category: Category,
        token_id: TokenId,
        power: u32,
        interest_rate: u32,
        duration: u32,
    ) {
        owner.require_auth();
        
        assert!(
            category == Category::Human,
            "Invalid Category to lend"
        );

        let mut nft = read_nft(&env.clone(), owner.clone(), category.clone(), token_id.clone());
        assert!(nft.locked_by_action == Action::None, "Card is locked by another action");
        
        nft.locked_by_action = Action::Lend;
        write_nft(&env.clone(), owner.clone(), category.clone(), token_id.clone(), nft);
        
        let lending_info = LendingInfo {
            lender: owner.clone(),
            category: category.clone(),
            token_id: token_id.clone(),
            power,
            interest_rate,
            duration,
            is_borrowed: false,
            borrower: owner.clone(),
            collateral_category: category.clone(),
            collateral_token_id: token_id.clone(),
            borrowed_block: 0,
        };

        add_lending(env.clone(), lending_info.clone());

        write_lending_info(env.clone(), owner.clone(), category.clone(), token_id.clone(), lending_info);
    }

    pub fn borrow(
        env: Env,
        borrower: Address,
        lender: Address,
        category: Category,
        token_id: TokenId,
        collateral_category: Category,
        collateral_token_id: TokenId,
    ) {
        borrower.require_auth();

        let mut nft = read_nft(&env.clone(), borrower.clone(), collateral_category.clone(), collateral_token_id.clone());
        assert!(nft.locked_by_action == Action::None, "Card is locked by another action");
        nft.locked_by_action = Action::Borrow;

        let mut lending_info = read_lending_info(env.clone(), lender.clone(), category.clone(), token_id.clone());
        assert!(lending_info.is_borrowed == false, "Card is already borrowed");
        assert!(nft.power > lending_info.power, "Collateral nft power must be equal or higher than the amount borrowed");

        lending_info.borrower = borrower;
        lending_info.collateral_category = collateral_category;
        lending_info.collateral_token_id = collateral_token_id;
        lending_info.is_borrowed = true;
        lending_info.borrowed_block = env.ledger().sequence();

        update_lending(env.clone(), lender.clone(), category.clone(), token_id.clone(), lending_info.clone());

        write_lending_info(env.clone(), lender.clone(), category.clone(), token_id.clone(), lending_info.clone());
    }

    pub fn read_lendings(env: Env) -> Vec<LendingInfo> {
        read_lendings(env.clone())
    }

    pub fn repay(
        env: Env,
        borrower: Address,
        lender: Address,
        category: Category,
        token_id: TokenId,
    ) {
        borrower.require_auth();
        let mut lending_info = read_lending_info(env.clone(), lender.clone(), category.clone(), token_id.clone());
        
    }
}
