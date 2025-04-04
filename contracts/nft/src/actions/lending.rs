use crate::{
    admin::{read_state, transfer_terry, write_state, State},
    *,
};
use admin::{read_balance, read_config, write_balance};
use nft_info::{read_nft, write_nft, Action, Category};
use soroban_sdk::{contracttype, vec, Address, Env, Vec};
use storage_types::{DataKey, TokenId, BALANCE_BUMP_AMOUNT, BALANCE_LIFETIME_THRESHOLD};
use user_info::{read_user, write_user};

const SCALE: u64 = 1_000_000; // 6-decimal fixed point
const APY_MIN: u64 = 0; // 0% APY
const APY_MAX: u64 = 300_000; // 30% APY = 0.30 * SCALE

#[contracttype]
#[derive(Clone, PartialEq)]
pub struct Lending {
    pub lender: Address,
    pub category: Category,
    pub token_id: TokenId,
    pub power: u32,
    pub lent_at: u64,
}

#[contracttype]
#[derive(Clone, PartialEq)]
pub struct Borrowing {
    pub borrower: Address,
    pub category: Category,
    pub token_id: TokenId,
    pub power: u32,
    pub borrowed_at: u64,
}

pub fn write_lending(
    env: Env,
    fee_payer: Address,
    category: Category,
    token_id: TokenId,
    lending: Lending,
) {
    let owner = read_user(&env, fee_payer).owner;

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

pub fn read_lending(
    env: Env,
    fee_payer: Address,
    category: Category,
    token_id: TokenId,
) -> Lending {
    let owner = read_user(&env, fee_payer).owner;

    let key = DataKey::Lending(owner.clone(), category.clone(), token_id.clone());
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
    env.storage().persistent().get(&key).unwrap()
}

pub fn remove_lending(env: Env, fee_payer: Address, category: Category, token_id: TokenId) {
    let owner = read_user(&env, fee_payer).owner;

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

pub fn write_borrowing(
    env: Env,
    fee_payer: Address,
    category: Category,
    token_id: TokenId,
    borrowing: Borrowing,
) {
    let owner = read_user(&env, fee_payer).owner;

    let key = DataKey::Borrowing(owner.clone(), category.clone(), token_id.clone());
    env.storage().persistent().set(&key, &borrowing);
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);

    let key = DataKey::Borrowings;
    let mut borrowings = read_borrowings(env.clone());
    if let Some(pos) = borrowings.iter().position(|borrowing| {
        borrowing.borrower == owner
            && borrowing.category == category
            && borrowing.token_id == token_id
    }) {
        borrowings.set(pos.try_into().unwrap(), borrowing)
    } else {
        borrowings.push_back(borrowing)
    }

    env.storage().persistent().set(&key, &borrowings);

    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

pub fn read_borrowing(
    env: Env,
    fee_payer: Address,
    category: Category,
    token_id: TokenId,
) -> Borrowing {
    fee_payer.require_auth();
    let owner = read_user(&env, fee_payer).owner;

    let key = DataKey::Borrowing(owner.clone(), category.clone(), token_id.clone());
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
    env.storage().persistent().get(&key).unwrap()
}

pub fn remove_borrowing(env: Env, fee_payer: Address, category: Category, token_id: TokenId) {
    let owner = read_user(&env, fee_payer).owner;

    let key = DataKey::Borrowing(owner.clone(), category.clone(), token_id.clone());
    env.storage().persistent().remove(&key);
    if env.storage().persistent().has(&key) {
        env.storage().persistent().extend_ttl(
            &key,
            BALANCE_LIFETIME_THRESHOLD,
            BALANCE_BUMP_AMOUNT,
        );
    }

    let key = DataKey::Borrowings;
    let mut borrowings = read_borrowings(env.clone());
    if let Some(pos) = borrowings.iter().position(|borrowing| {
        borrowing.borrower == owner
            && borrowing.category == category
            && borrowing.token_id == token_id
    }) {
        borrowings.remove(pos.try_into().unwrap());
    }

    env.storage().persistent().set(&key, &borrowings);

    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

pub fn read_borrowings(env: Env) -> Vec<Borrowing> {
    let key = DataKey::Borrowings;
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or(vec![&env.clone()])
}

pub fn calculate_apy(
    total_demand: u64,
    total_offer: u64,
    total_loan_duration: u64,
    total_loan_count: u64,
    alpha: u64,
) -> u64 {
    if total_demand == 0 || total_offer == 0 || total_loan_count == 0 {
        return APY_MIN;
    }
    let demand_ratio = total_demand * SCALE / total_offer;
    let average_duration = total_loan_duration * SCALE / total_loan_count;
    let alpha_t = alpha * average_duration / SCALE;
    let time_denom = SCALE + alpha_t;
    let time_factor = SCALE * SCALE / time_denom;
    let multiplier = demand_ratio * time_factor / SCALE;
    let apy_range = APY_MAX - APY_MIN;
    let apy = APY_MIN + (apy_range * multiplier / SCALE);
    apy
}

fn calculate_interest(principal: u64, apy: u64, loan_duration: u64) -> u64 {
    principal * apy * loan_duration / 8_760 / SCALE
}

pub fn lend(env: Env, fee_payer: Address, category: Category, token_id: TokenId, power: u32) {
    fee_payer.require_auth();
    let owner = read_user(&env, fee_payer).owner;

    assert!(
        category == Category::Resource || category == Category::Leader,
        "Invalid Category to lend"
    );

    let mut nft = read_nft(&env.clone(), owner.clone(), token_id.clone()).unwrap();
    assert!(
        nft.locked_by_action == Action::None,
        "Card is locked by another action"
    );
    assert!(nft.power >= power, "Exceed power amount to lend");

    nft.locked_by_action = Action::Lend;

    write_nft(&env.clone(), owner.clone(), token_id.clone(), nft);

    let config = read_config(&env);
    let mut balance = read_balance(&env);

    let power_fee = power * config.power_action_fee / 100;
    balance.haw_ai_power += power_fee;

    write_balance(&env, &balance);

    let mut state = read_state(&env);

    state.total_offer += power as u64;

    write_state(&env, &state);

    transfer_terry(&env, owner.clone(), config.terry_per_action);

    let lending = Lending {
        lender: owner.clone(),
        category: category.clone(),
        token_id: token_id.clone(),
        power,
        lent_at: env.ledger().timestamp(),
    };

    write_lending(
        env.clone(),
        owner.clone(),
        category.clone(),
        token_id.clone(),
        lending,
    );
}

pub fn borrow(env: Env, fee_payer: Address, category: Category, token_id: TokenId, power: u32) {
    fee_payer.require_auth();
    let mut user = read_user(&env, fee_payer.clone());
    let owner = user.owner.clone();

    assert!(
        category == Category::Resource || category == Category::Leader,
        "Invalid Category to borrow"
    );

    let mut nft = read_nft(&env.clone(), owner.clone(), token_id.clone()).unwrap();
    assert!(
        nft.locked_by_action == Action::None,
        "Card is locked by another action"
    );

    let config = read_config(&env);

    let state = read_state(&env);

    let apy = calculate_apy(
        state.total_demand,
        state.total_offer,
        state.total_loan_duration,
        state.total_loan_count,
        config.apy_alpha as u64,
    );
    let reserve = power as u64 + calculate_interest(power as u64, apy, 4_380);
    assert!(nft.power as u64 >= reserve, "Exceed power amount to borrow");

    nft.locked_by_action = Action::Borrow;

    write_nft(&env, owner.clone(), token_id.clone(), nft);

    let mut balance = read_balance(&env);

    let power_fee = power * config.power_action_fee / 100;
    balance.haw_ai_power += power_fee;

    write_balance(&env, &balance);

    user.power += power;

    write_user(&env.clone(), owner.clone(), user);

    transfer_terry(&env, owner.clone(), config.terry_per_action);

    let borrowing = Borrowing {
        borrower: owner.clone(),
        category: category.clone(),
        token_id: token_id.clone(),
        power: power.clone(),
        borrowed_at: env.ledger().timestamp(),
    };

    write_borrowing(
        env.clone(),
        owner.clone(),
        category.clone(),
        token_id.clone(),
        borrowing,
    );
}

pub fn repay(env: Env, fee_payer: Address, category: Category, token_id: TokenId) {
    fee_payer.require_auth();
    let mut user = read_user(&env, fee_payer);
    let owner = user.owner.clone();

    assert!(
        category == Category::Resource || category == Category::Leader,
        "Invalid Category to repay"
    );

    let mut nft = read_nft(&env.clone(), owner.clone(), token_id.clone()).unwrap();
    assert!(
        nft.locked_by_action == Action::Borrow,
        "Card is not locked by borrow action"
    );

    let borrowing = read_borrowing(
        env.clone(),
        owner.clone(),
        category.clone(),
        token_id.clone(),
    );

    let config = read_config(&env);

    let mut state = read_state(&env);

    let loan_duration = (env.ledger().timestamp() - borrowing.borrowed_at) / 3_600;
    state.total_demand += borrowing.power as u64 * loan_duration;
    state.total_loan_duration += loan_duration;
    state.total_loan_count += 1;

    let apy = calculate_apy(
        state.total_demand,
        state.total_offer,
        state.total_loan_duration,
        state.total_loan_count,
        config.apy_alpha as u64,
    );
    let interest_amount = calculate_interest(borrowing.power as u64, apy, loan_duration);
    state.total_interest += interest_amount as u64;

    write_state(&env, &state);

    nft.locked_by_action = Action::None;

    write_nft(&env, owner.clone(), token_id.clone(), nft);

    assert!(
        user.power >= borrowing.power + interest_amount as u32,
        "Insufficient fund to repay",
    );

    user.power -= borrowing.power + interest_amount as u32;

    write_user(&env, owner.clone(), user);

    let config = read_config(&env);

    transfer_terry(&env, owner.clone(), config.terry_per_action);

    remove_borrowing(env, owner, category, token_id);
}

fn liquidate(env: Env, state: &mut State) {
    for borrowing in read_borrowings(env.clone()) {
        let nft = read_nft(&env, borrowing.borrower.clone(), borrowing.token_id.clone()).unwrap();
        state.total_interest += nft.power as u64;
        remove_borrowing(
            env.clone(),
            borrowing.borrower,
            borrowing.category,
            borrowing.token_id,
        );
    }
}

pub fn withdraw(env: Env, fee_payer: Address, category: Category, token_id: TokenId) {
    fee_payer.require_auth();
    let mut user = read_user(&env, fee_payer);
    let owner = user.owner.clone();

    assert!(
        category == Category::Resource || category == Category::Leader,
        "Invalid Category to withdraw"
    );

    let mut nft = read_nft(&env.clone(), owner.clone(), token_id.clone()).unwrap();
    assert!(
        nft.locked_by_action == Action::Lend,
        "Card is not locked by lend action"
    );

    let lending = read_lending(
        env.clone(),
        owner.clone(),
        category.clone(),
        token_id.clone(),
    );

    let config = read_config(&env);

    let mut state = read_state(&env);

    let loan_duration = (env.ledger().timestamp() - lending.lent_at) / 3_600;
    let apy = calculate_apy(
        state.total_demand,
        state.total_offer,
        state.total_loan_duration,
        state.total_loan_count,
        config.apy_alpha as u64,
    );
    let interest_amount = calculate_interest(lending.power as u64, apy, loan_duration);

    if state.total_interest < interest_amount {
        liquidate(env.clone(), &mut state);
    }

    state.total_interest -= interest_amount;

    write_state(&env, &state);

    nft.locked_by_action = Action::None;

    write_nft(&env, owner.clone(), token_id.clone(), nft);

    user.power += interest_amount as u32;

    write_user(&env, owner.clone(), user);

    let config = read_config(&env);

    transfer_terry(&env, owner.clone(), config.terry_per_action);

    remove_lending(env, owner, category, token_id);
}
