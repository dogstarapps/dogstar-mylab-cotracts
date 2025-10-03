use crate::{
    admin::{read_state, write_state},
    user_info::mint_terry,
    *,
};
use admin::{read_balance, read_config, write_balance};
use nft_info::{read_nft, write_nft, Action, Category};
use soroban_sdk::{contracttype, symbol_short, vec, Address, Env, Vec};
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
    user: Address,
    category: Category,
    token_id: TokenId,
    lending: Lending,
) {
    let owner = read_user(&env, user).owner;

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

pub fn read_lending(env: Env, user: Address, category: Category, token_id: TokenId) -> Lending {
    let owner = read_user(&env, user).owner;

    let key = DataKey::Lending(owner.clone(), category.clone(), token_id.clone());
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
    env.storage().persistent().get(&key).unwrap()
}

pub fn remove_lending(env: Env, user: Address, category: Category, token_id: TokenId) {
    let owner = read_user(&env, user).owner;

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
    user: Address,
    category: Category,
    token_id: TokenId,
    borrowing: Borrowing,
) {
    let owner = read_user(&env, user).owner;

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
        borrowings.set(pos.try_into().unwrap(), borrowing.clone())
    } else {
        borrowings.push_back(borrowing.clone())
    }

    env.storage().persistent().set(&key, &borrowings);

    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
    env.events().publish(
        (symbol_short!("borrowing"), symbol_short!("open")),
        borrowing,
    );
}

pub fn read_borrowing(env: Env, user: Address, category: Category, token_id: TokenId) -> Borrowing {
    user.require_auth();
    let owner = read_user(&env, user).owner;

    let key = DataKey::Borrowing(owner.clone(), category.clone(), token_id.clone());
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
    env.storage().persistent().get(&key).unwrap()
}

pub fn remove_borrowing(env: Env, user: Address, category: Category, token_id: TokenId) {
    let owner = read_user(&env, user.clone()).owner;

    // let key = DataKey::Borrowing(owner.clone(), category.clone(), token_id.clone());
    // env.storage().persistent().remove(&key);
    // if env.storage().persistent().has(&key) {
    //     env.storage().persistent().extend_ttl(
    //         &key,
    //         BALANCE_LIFETIME_THRESHOLD,
    //         BALANCE_BUMP_AMOUNT,
    //     );
    // }

    let key = DataKey::Borrowings;
    let mut borrowings = read_borrowings(env.clone());
    if let Some(pos) = borrowings.iter().position(|borrowing| {
        borrowing.borrower == owner
            && borrowing.category == category
            && borrowing.token_id == token_id
    }) {
        let borrowing = read_borrowing(
            env.clone(),
            user.clone(),
            category.clone(),
            token_id.clone(),
        );
        env.events().publish(
            (symbol_short!("borrowing"), symbol_short!("close")),
            borrowing.clone(),
        );
        borrowings.remove(pos.try_into().unwrap());
    }

    env.storage().persistent().set(&key, &borrowings);

    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);

    let key = DataKey::Borrowing(owner.clone(), category.clone(), token_id.clone());
    env.storage().persistent().remove(&key);
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

pub fn lend(env: Env, user: Address, category: Category, token_id: TokenId, power: u32) {
    user.require_auth();
    let owner = read_user(&env, user).owner;
    let config = read_config(&env);
    let power_fee: u32 = power.saturating_mul(config.power_action_fee) / 100;
    let lend_amount: u32 = power.saturating_sub(power_fee);
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

    let mut balance = read_balance(&env);

    balance.haw_ai_power += power_fee;

    let mut state = read_state(&env);

    state.total_offer += power as u64;

    write_state(&env, &state);

    let lending = Lending {
        lender: owner.clone(),
        category: category.clone(),
        token_id: token_id.clone(),
        power: lend_amount,
        lent_at: env.ledger().timestamp(),
    };

    write_lending(
        env.clone(),
        owner.clone(),
        category.clone(),
        token_id.clone(),
        lending,
    );

    // Mint terry to user as rewards
    mint_terry(&env, owner.clone(), config.terry_per_lending);

    balance.haw_ai_terry += config.terry_per_lending * config.haw_ai_percentage as i128 / 100;
    write_balance(&env, &balance);
}

pub fn borrow(env: Env, user: Address, category: Category, token_id: TokenId, power: u32) {
    user.require_auth();
    let mut user = read_user(&env, user.clone());
    let owner = user.owner.clone();
    let config = read_config(&env);
    let power_fee: u32 = power.saturating_mul(config.power_action_fee) / 100;
    let borrow_amount: u32 = power.saturating_sub(power_fee);

    assert!(
        category == Category::Resource || category == Category::Leader,
        "Invalid Category to borrow"
    );

    let mut nft = read_nft(&env.clone(), owner.clone(), token_id.clone()).unwrap();
    assert!(
        nft.locked_by_action == Action::None,
        "Card is locked by another action"
    );

    // config already read above

    let mut state = read_state(&env);
    assert!(
        state.total_offer >= power as u64,
        "Insufficient power to borrow"
    );

    state.total_offer -= borrow_amount as u64;
    state.total_borrowed_power += borrow_amount as u64;

    write_state(&env, &state);

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


    balance.haw_ai_power += power_fee;

    user.power += borrow_amount;

    write_user(&env.clone(), owner.clone(), user);

    let borrowing = Borrowing {
        borrower: owner.clone(),
        category: category.clone(),
        token_id: token_id.clone(),
        power: borrow_amount,
        borrowed_at: env.ledger().timestamp(),
    };

    write_borrowing(
        env.clone(),
        owner.clone(),
        category.clone(),
        token_id.clone(),
        borrowing,
    );

    // Mint terry to user as rewards
    mint_terry(&env, owner.clone(), config.terry_per_lending);

    balance.haw_ai_terry += config.terry_per_lending * config.haw_ai_percentage as i128 / 100;
    write_balance(&env, &balance);
}

pub fn repay(env: Env, user: Address, category: Category, token_id: TokenId) {
    user.require_auth();
    let mut user = read_user(&env, user);
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
    state.total_offer += borrowing.power as u64;
    state.total_borrowed_power -= borrowing.power as u64;

    write_state(&env, &state);

    nft.locked_by_action = Action::None;

    write_nft(&env, owner.clone(), token_id.clone(), nft);

    assert!(
        user.power >= borrowing.power + interest_amount as u32,
        "Insufficient fund to repay"
    );

    user.power -= borrowing.power + interest_amount as u32;

    write_user(&env, owner.clone(), user);

    remove_borrowing(env.clone(), owner.clone(), category, token_id);

    // Mint terry to user as rewards
    let config = read_config(&env);
    mint_terry(&env, owner.clone(), config.terry_per_lending);

    let mut balance = read_balance(&env);
    balance.haw_ai_terry += config.terry_per_lending * config.haw_ai_percentage as i128 / 100;
    write_balance(&env, &balance);
}

fn check_liquidations(env: Env) {
    for borrowing in read_borrowings(env.clone()) {
        let nft = read_nft(&env, borrowing.borrower.clone(), borrowing.token_id.clone()).unwrap();

        let config = read_config(&env);
        let mut state = read_state(&env);
        let apy = calculate_apy(
            state.total_demand,
            state.total_offer,
            state.total_loan_duration,
            state.total_loan_count,
            config.apy_alpha as u64,
        );
        let loan_duration = (env.ledger().timestamp() - borrowing.borrowed_at) / 3_600;
        let interest_amount = calculate_interest(borrowing.power as u64, apy, loan_duration);

        if nft.power < borrowing.power + interest_amount as u32 {
            state.total_interest += nft.power as u64;

            write_state(&env, &state);
            remove_borrowing(
                env.clone(),
                borrowing.borrower,
                borrowing.category,
                borrowing.token_id,
            );
        }
    }
}

fn liquidate(env: Env, user: Address, category: Category, token_id: TokenId) {
    user.require_auth();
    let user = read_user(&env, user);
    let owner = user.owner.clone();

    let borrowing = read_borrowing(
        env.clone(),
        owner.clone(),
        category.clone(),
        token_id.clone(),
    );
    if borrowing.borrower == owner {
        let nft = read_nft(&env, borrowing.borrower.clone(), borrowing.token_id.clone()).unwrap();

        let config = read_config(&env);
        let mut state = read_state(&env);
        let apy = calculate_apy(
            state.total_demand,
            state.total_offer,
            state.total_loan_duration,
            state.total_loan_count,
            config.apy_alpha as u64,
        );
        let loan_duration = (env.ledger().timestamp() - borrowing.borrowed_at) / 3_600;
        let interest_amount = calculate_interest(borrowing.power as u64, apy, loan_duration);

        if nft.power < borrowing.power + interest_amount as u32 {
            state.total_interest += nft.power as u64;

            write_state(&env, &state);
            remove_borrowing(
                env.clone(),
                borrowing.borrower,
                borrowing.category,
                borrowing.token_id,
            );
        }
    }
}

pub fn withdraw(env: Env, user: Address, category: Category, token_id: TokenId) {
    user.require_auth();
    let mut user = read_user(&env, user);
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
        check_liquidations(env.clone());
    }

    state = read_state(&env);

    if state.total_interest >= interest_amount {
      state.total_interest -= interest_amount;
    } else {
        state.total_interest = 0;
    }
    state.total_offer -= lending.power as u64;

    write_state(&env, &state);

    nft.locked_by_action = Action::None;

    write_nft(&env, owner.clone(), token_id.clone(), nft);

    user.power += interest_amount as u32;

    write_user(&env, owner.clone(), user);

    remove_lending(env.clone(), owner.clone(), category, token_id);

    // Mint terry to user as rewards
    let config = read_config(&env);
    mint_terry(&env, owner.clone(), config.terry_per_lending);

    let mut balance = read_balance(&env);
    balance.haw_ai_terry += config.terry_per_lending * config.haw_ai_percentage as i128 / 100;
    write_balance(&env, &balance);
}

pub fn get_current_apy(env: Env) -> u64 {
    let config = read_config(&env);

    let state = read_state(&env);

    calculate_apy(
        state.total_demand,
        state.total_offer,
        state.total_loan_duration,
        state.total_loan_count,
        config.apy_alpha as u64,
    )
}
