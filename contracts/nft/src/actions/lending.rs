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
use crate::event::{emit_lend, emit_borrow, emit_withdraw, emit_repay};

const SCALE: u64 = 1_000_000; // 6-decimal fixed point
const APY_MIN: u64 = 0; // 0% APY
const APY_MAX: u64 = 300_000; // 30% APY = 0.30 * SCALE
const T_MAX_FP: u64 = 500_000; // 0.5 years in SCALE for reserve horizon

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
    // Avoid divisions by zero and clamp utilization/time factors
    // Utilization proxy based on current aggregates; add small epsilon to offer
    let offer_eps = total_offer.saturating_add(1);
    let mut demand_ratio = (total_demand as u128)
        .saturating_mul(SCALE as u128)
        / (offer_eps as u128);
    if demand_ratio > SCALE as u128 {
        demand_ratio = SCALE as u128;
    }

    // Average duration with a minimal floor (1 hour in the same units used by the state)
    let average_duration = if total_loan_count == 0 {
        SCALE // 1 hour in fixed point
    } else {
        total_loan_duration.saturating_mul(SCALE) / total_loan_count
    };

    let alpha_t = (alpha as u128)
        .saturating_mul(average_duration as u128)
        / (SCALE as u128);
    let time_denom = (SCALE as u128).saturating_add(alpha_t);
    let time_factor = ((SCALE as u128) * (SCALE as u128)) / time_denom;
    let multiplier = (demand_ratio.saturating_mul(time_factor)) / (SCALE as u128);
    let apy_range = (APY_MAX - APY_MIN) as u128;
    let mut apy = (APY_MIN as u128).saturating_add((apy_range.saturating_mul(multiplier)) / (SCALE as u128));
    if apy > APY_MAX as u128 {
        apy = APY_MAX as u128;
    }
    apy as u64
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

    // Move gross power out of the card; fee goes to pot, net to pool (state.offer)
    nft.power = nft.power.saturating_sub(power);
    nft.locked_by_action = Action::Lend;
    write_nft(&env.clone(), owner.clone(), token_id.clone(), nft);

    let mut balance = read_balance(&env);

    balance.haw_ai_power += power_fee;

    let mut state = read_state(&env);

    state.total_offer += lend_amount as u64; // principal_net supplied to pool

    write_state(&env, &state);

    let lent_at = env.ledger().timestamp();
    let lending = Lending {
        lender: owner.clone(),
        category: category.clone(),
        token_id: token_id.clone(),
        power: lend_amount,
        lent_at,
    };

    write_lending(
        env.clone(),
        owner.clone(),
        category.clone(),
        token_id.clone(),
        lending,
    );

    // Emit lend event
    emit_lend(&env, &owner);

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

    // Borrow > 0 validations
    assert!(power > 0, "Invalid borrow: zero");
    assert!(borrow_amount > 0, "Invalid borrow: net <= 0");

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
    assert!(state.total_offer >= borrow_amount as u64, "Insufficient power to borrow");

    // Tentatively update pool before APY to compute post-borrow state
    state.total_offer -= borrow_amount as u64;
    state.total_borrowed_power += borrow_amount as u64;
    write_state(&env, &state);

    // Compute APY and reserve using horizon T_MAX and guard k<1
    let apy = calculate_apy(
        state.total_demand,
        state.total_offer,
        state.total_loan_duration,
        state.total_loan_count,
        config.apy_alpha as u64,
    );

    // k = APY * T_MAX in fixed point
    let k_fp = (apy as u128)
        .saturating_mul(T_MAX_FP as u128)
        / (SCALE as u128);
    assert!(k_fp < SCALE as u128, "Invalid horizon: APY*T_max >= 1");
    // Reserve = P * k / (1 - k)
    let reserve = (borrow_amount as u128)
        .saturating_mul(k_fp)
        / ((SCALE as u128).saturating_sub(k_fp));

    // Fee and buffer checks
    let buffer_bps: u32 = 500; // 5% default safety buffer
    let collateral_net = (nft.power as u128).saturating_sub(power_fee as u128);
    let buffer = (collateral_net.saturating_mul(buffer_bps as u128)) / 10_000u128;
    let lhs = (borrow_amount as u128)
        .saturating_add(reserve)
        .saturating_add(power_fee as u128)
        .saturating_add(buffer);
    assert!(lhs <= nft.power as u128, "Exceeds collateral capacity");

    nft.locked_by_action = Action::Borrow;

    // Deduct fee immediately from collateral card and lock
    nft.power = nft.power.saturating_sub(power_fee);
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

    // Emit borrow event
    emit_borrow(&env, &owner);

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

    // Emit repay event
    emit_repay(&env, &owner);

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

    // Return principal_net to the same card and unlock
    nft.power = nft.power.saturating_add(lending.power);
    nft.locked_by_action = Action::None;
    write_nft(&env, owner.clone(), token_id.clone(), nft);

    let power_fee: u32 =
        (interest_amount.saturating_mul(config.power_action_fee as u64) / 100) as u32;
    let reward_interest: u64 = interest_amount.saturating_sub(power_fee as u64);

    user.power += reward_interest as u32;

    write_user(&env, owner.clone(), user);

    // Emit withdraw event
    emit_withdraw(&env, &owner);

    remove_lending(env.clone(), owner.clone(), category, token_id);

    // Mint terry to user as rewards
    let config = read_config(&env);
    mint_terry(&env, owner.clone(), config.terry_per_lending);

    let mut balance = read_balance(&env);
    balance.haw_ai_power += power_fee;
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
