use crate::*;
use admin::{read_balance, read_config, write_balance};
use nft_info::{read_nft, write_nft, Action, Category};
use soroban_sdk::{contracttype, vec, Address, Env, Vec};
use storage_types::{DataKey, TokenId, BALANCE_BUMP_AMOUNT, BALANCE_LIFETIME_THRESHOLD};
// use user::{read_user, write_user};

#[contracttype]
#[derive(Clone, PartialEq)]
pub struct Lending {
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

pub fn write_lending(
    env: Env,
    fee_payer: Address,
    category: Category,
    token_id: TokenId,
    lending: Lending,
) {
    fee_payer.require_auth();
    let owner = read_user_by_fee_payer(e, fee_payer).owner;

    let key = DataKey::Lending(owner.clone(), category.clone(), token_id.clone());
    env.storage().persistent().set(&key, &lending);
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);

    let key = DataKey::Lendings;
    let mut lendings = read_lendings(env.clone());
    if let Some(pos) = lendings.iter().position(|lending| {
        lending.lender == owner && lending.category == category && lending.token_id == token_id
    }) {
        lendings.set(pos.try_into().unwrap(), lending)
    } else {
        lendings.push_back(lending)
    }

    env.storage().persistent().set(&key, &lendings);

    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

pub fn read_lending(env: Env, fee_payer: Address, category: Category, token_id: TokenId) -> Lending {
    fee_payer.require_auth();
    let owner = read_user_by_fee_payer(e, fee_payer).owner;

    let key = DataKey::Lending(owner.clone(), category.clone(), token_id.clone());
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
    env.storage().persistent().get(&key).unwrap()
}

pub fn remove_lending(env: Env, fee_payer: Address, category: Category, token_id: TokenId) {
    fee_payer.require_auth();
    let owner = read_user_by_fee_payer(e, fee_payer).owner;

    let key = DataKey::Lending(owner.clone(), category.clone(), token_id.clone());
    env.storage().persistent().remove(&key);
    if env.storage().persistent().has(&key) {
        env.storage().persistent().extend_ttl(
            &key,
            BALANCE_LIFETIME_THRESHOLD,
            BALANCE_BUMP_AMOUNT,
        );
    }

    let key = DataKey::Lendings;
    let mut lendings = read_lendings(env.clone());
    if let Some(pos) = lendings.iter().position(|lending| {
        lending.lender == owner && lending.category == category && lending.token_id == token_id
    }) {
        lendings.remove(pos.try_into().unwrap());
    }

    env.storage().persistent().set(&key, &lendings);

    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

pub fn read_lendings(env: Env) -> Vec<Lending> {
    let key = DataKey::Lendings;
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or(vec![&env.clone()])
}

pub fn lend(
    env: Env,
    fee_payer: Address,
    category: Category,
    token_id: TokenId,
    power: u32,
    interest_rate: u32,
    duration: u32,
) {
    fee_payer.require_auth();
    let owner = read_user_by_fee_payer(e, fee_payer).owner;

    assert!(category == Category::Human, "Invalid Category to lend");

    let mut nft = read_nft(
        &env.clone(),
        owner.clone(),
        category.clone(),
        token_id.clone(),
    );
    assert!(
        nft.locked_by_action == Action::None,
        "Card is locked by another action"
    );
    assert!(nft.power >= power, "Exceed power amount to lend");

    nft.locked_by_action = Action::Lend;

    write_nft(
        &env.clone(),
        owner.clone(),
        category.clone(),
        token_id.clone(),
        nft,
    );

    let lending = Lending {
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

    write_lending(
        env.clone(),
        owner.clone(),
        category.clone(),
        token_id.clone(),
        lending,
    );
}

pub fn borrow(
    env: Env,
    fee_payer: Address,
    lender: Address,
    category: Category,
    token_id: TokenId,
    collateral_category: Category,
    collateral_token_id: TokenId,
) {
    fee_payer.require_auth();
    let borrower = read_user_by_fee_payer(e, fee_payer).owner;


    let mut borrower_nft = read_nft(
        &env.clone(),
        borrower.clone(),
        collateral_category.clone(),
        collateral_token_id.clone(),
    );
    let mut lender_nft = read_nft(
        &env.clone(),
        lender.clone(),
        category.clone(),
        token_id.clone(),
    );

    assert!(
        borrower_nft.locked_by_action == Action::None,
        "Card is locked by another action"
    );

    let mut lending = read_lending(
        env.clone(),
        lender.clone(),
        category.clone(),
        token_id.clone(),
    );

    let config = read_config(&env);
    let power_fee = config.power_action_fee * lending.power / 100;

    let mut balance = read_balance(&env);
    balance.haw_ai_power += power_fee;
    write_balance(&env, &balance);

    borrower_nft.locked_by_action = Action::Borrow;
    borrower.power += lending.power;
    borrower.power -= power_fee;;

    //borrower_nft.power += lending.power;
    //borrower_nft.power -= power_fee;

    assert!(lending.is_borrowed == false, "Card is already borrowed");
    assert!(
        borrower.power > lending.power,
        "Collateral nft power must be equal or higher than the amount borrowed"
    );

    write_nft(
        &env,
        borrower.clone(),
        collateral_category.clone(),
        collateral_token_id.clone(),
        borrower_nft,
    );

    lender_nft.power -= lending.power;

    write_nft(
        &env,
        lender.clone(),
        category.clone(),
        token_id.clone(),
        lender_nft,
    );

    lending.borrower = borrower.clone();
    lending.collateral_category = collateral_category;
    lending.collateral_token_id = collateral_token_id;
    lending.is_borrowed = true;
    lending.borrowed_block = env.ledger().sequence();

    // let mut user = read_user(&env.clone(), borrower.clone());
    // user.borrowed_power += lending.power;
    // write_user(&env.clone(), borrower.clone(), user);

    write_lending(
        env.clone(),
        lender.clone(),
        category.clone(),
        token_id.clone(),
        lending.clone(),
    );
}

pub fn lendings(env: Env) -> Vec<Lending> {
    read_lendings(env.clone())
}

pub fn repay(env: Env, fee_payer: Address, lender: Address, category: Category, token_id: TokenId) {
    fee_payer.require_auth();
    let borrower = read_user_by_fee_payer(e, fee_payer).owner;

    let mut lending = read_lending(
        env.clone(),
        lender.clone(),
        category.clone(),
        token_id.clone(),
    );
    let current_block = env.ledger().sequence();
    let time_elapsed = current_block - lending.borrowed_block;
    let interest_amount =
        lending.power * lending.interest_rate * time_elapsed * 100 / lending.duration;

    let mut lender_nft = read_nft(
        &env.clone(),
        lender.clone(),
        category.clone(),
        token_id.clone(),
    );
    let mut borrower_nft = read_nft(
        &env.clone(),
        borrower.clone(),
        lending.collateral_category.clone(),
        lending.collateral_token_id.clone(),
    );

    lender_nft.power += interest_amount;

    borrower_nft.power -= interest_amount;
    borrower_nft.locked_by_action = Action::None;

    write_nft(
        &env.clone(),
        lender.clone(),
        category.clone(),
        token_id.clone(),
        lender_nft,
    );

    write_nft(
        &env.clone(),
        borrower.clone(),
        lending.collateral_category.clone(),
        lending.collateral_token_id.clone(),
        borrower_nft,
    );

    lending.is_borrowed = false;

    write_lending(
        env.clone(),
        lender.clone(),
        category.clone(),
        token_id.clone(),
        lending,
    );
}

pub fn withdraw(env: Env, fee_payer: Address, category: Category, token_id: TokenId) {
    fee_payer.require_auth();
    let lender = read_user_by_fee_payer(e, fee_payer).owner;

    let lending = read_lending(
        env.clone(),
        lender.clone(),
        category.clone(),
        token_id.clone(),
    );

    let current_block = env.ledger().sequence();

    assert!(
        lending.is_borrowed == false || lending.borrowed_block + lending.duration <= current_block,
        "Borrowed Duration"
    );

    if lending.is_borrowed {
        let current_block = env.ledger().sequence();
        let time_elapsed = current_block - lending.borrowed_block;
        let interest_amount =
            lending.power * lending.interest_rate * time_elapsed * 100 / lending.duration;

        let mut lender_nft = read_nft(
            &env.clone(),
            lender.clone(),
            category.clone(),
            token_id.clone(),
        );
        let mut borrower_nft = read_nft(
            &env.clone(),
            lending.borrower.clone(),
            lending.collateral_category.clone(),
            lending.collateral_token_id.clone(),
        );

        lender_nft.power += interest_amount;

        borrower_nft.power -= interest_amount;
        borrower_nft.locked_by_action = Action::None;

        write_nft(
            &env.clone(),
            lender.clone(),
            category.clone(),
            token_id.clone(),
            lender_nft,
        );

        write_nft(
            &env.clone(),
            lending.borrower.clone(),
            lending.collateral_category.clone(),
            lending.collateral_token_id.clone(),
            borrower_nft,
        );
    }

    remove_lending(
        env.clone(),
        lender.clone(),
        category.clone(),
        token_id.clone(),
    );
}
