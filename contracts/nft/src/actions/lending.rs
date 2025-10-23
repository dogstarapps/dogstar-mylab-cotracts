use crate::{
    admin::{read_state, write_state},
    user_info::mint_terry,
    *,
};
use admin::{read_balance, read_config, write_balance};
use nft_info::{read_nft, write_nft, Action, Category};
use soroban_sdk::{contracttype, vec, Address, Env, Vec};
use storage_types::{DataKey, TokenId, State, BALANCE_BUMP_AMOUNT, BALANCE_LIFETIME_THRESHOLD};
use user_info::{read_user, write_user};
use crate::event::{
    emit_lend, emit_borrow, emit_withdraw, emit_repay, emit_lend_deposited,
    emit_borrow_opened, emit_withdraw_paid, emit_index_updated, emit_loan_touched,
    emit_card_locked, emit_card_unlocked, emit_fee_to_hawaii, emit_terry_awarded, emit_apy_updated
};
use crate::pot::management::accumulate_pot_internal;

const SCALE: u64 = 1_000_000; // 6-decimal fixed point
const APY_MIN: u64 = 0; // 0% APY
const APY_MAX: u64 = 300_000; // 30% APY = 0.30 * SCALE

#[contracttype]
#[derive(Clone, PartialEq)]
pub struct Lending {
    pub lender: Address,
    pub category: Category,
    pub token_id: TokenId,
    pub principal_net: u32, // Net principal after fee deduction
    pub lent_at: u64,
}

#[contracttype]
#[derive(Clone, PartialEq)]
pub struct Borrowing {
    pub borrower: Address,
    pub category: Category,
    pub token_id: TokenId,
    pub principal: u32, // Principal borrowed (net after fee)
    pub reserve: u64, // Reserve escrowed from collateral
    pub collateral_power: u32, // Original collateral POWER
    pub borrowed_at: u64,
    pub last_liquidation_index: u64, // lastL for tracking haircuts
    pub weight: u64, // Weight for pro-rata (reserve_remaining)
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
    let owner = read_user(&env, user).owner;

    let key = DataKey::Borrowing(owner.clone(), category.clone(), token_id.clone());
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
    env.storage().persistent().get(&key).unwrap()
}

pub fn remove_borrowing(env: Env, user: Address, category: Category, token_id: TokenId) {
    let owner = read_user(&env, user.clone()).owner;

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
    // Per spec: "If no one borrows, the APY is 0"
    if total_demand == 0 || total_offer == 0 || total_loan_count == 0 {
        return APY_MIN; // 0%
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

fn calculate_reserve(principal: u64, apy: u64, t_max: u64) -> u64 {
    let apy_t = apy * t_max / SCALE;
    assert!(apy_t < SCALE, "APY × T_max must be < 1");
    let numerator = principal * apy_t;
    let denominator = SCALE - apy_t;
    numerator / denominator
}

fn calculate_max_borrow(collateral: u32, apy: u64, t_max: u64, fee_bps: u32, buffer_bps: u32) -> u64 {
    let apy_t = apy * t_max / SCALE;
    assert!(apy_t < SCALE, "APY × T_max must be < 1");


    let buffer = (collateral as u64 * buffer_bps as u64) / 10000;
    let available = (collateral as u64).saturating_sub(buffer);

    let factor = SCALE - apy_t;
    let numerator = available * factor;
    let fee_factor = 10000 + fee_bps as u64;
    let denominator = SCALE * fee_factor / 10000;

    numerator / denominator
}

pub fn lend(env: Env, user: Address, category: Category, token_id: TokenId, power: u32) {
    user.require_auth();
    let owner = read_user(&env, user).owner;
    let config = read_config(&env);

    // Calculate fee from card POWER
    let power_fee: u32 = (power as u64 * config.fee_lend_bps as u64 / 10000) as u32;
    let principal_net: u32 = power.saturating_sub(power_fee);

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

    // Deduct full power (including fee) from card
    nft.power = nft.power.saturating_sub(power);
    nft.locked_by_action = Action::Lend;
    write_nft(&env.clone(), owner.clone(), token_id.clone(), nft);

    accumulate_pot_internal(&env, 0, power_fee, 0, Some(owner.clone()), Some(Action::Lend));
    emit_fee_to_hawaii(&env, power_fee, Action::Lend);

    let mut state = read_state(&env);
    state.total_offer += principal_net as u64;
    write_state(&env, &state);

    let lent_at = env.ledger().timestamp();
    let lending = Lending {
        lender: owner.clone(),
        category: category.clone(),
        token_id: token_id.clone(),
        principal_net,
        lent_at,
    };

    write_lending(
        env.clone(),
        owner.clone(),
        category.clone(),
        token_id.clone(),
        lending,
    );

    emit_lend(&env, &owner);
    emit_lend_deposited(&env, &owner, power, power_fee, principal_net);
    emit_card_locked(&env, &owner, Action::Lend);

    // Mint terry to user as rewards
    mint_terry(&env, owner.clone(), config.terry_per_lending);
    emit_terry_awarded(&env, &owner, config.terry_per_lending, Action::Lend, config.terry_per_lending);

    let mut balance = read_balance(&env);
    balance.haw_ai_terry += config.terry_per_lending * config.haw_ai_percentage as i128 / 100;
    write_balance(&env, &balance);
}

pub fn borrow(env: Env, user: Address, category: Category, token_id: TokenId, power_requested: u32) {
    user.require_auth();
    let mut user = read_user(&env, user.clone());
    let owner = user.owner.clone();
    let config = read_config(&env);

    assert!(
        category == Category::Resource || category == Category::Leader,
        "Invalid Category to borrow"
    );

    let mut nft = read_nft(&env.clone(), owner.clone(), token_id.clone()).unwrap();
    assert!(
        nft.locked_by_action == Action::None,
        "Card is locked by another action"
    );

    let collateral_power = nft.power;

    let mut state = read_state(&env);

    // Calculate current APY
    let apy = calculate_apy(
        state.total_demand,
        state.total_offer,
        state.total_loan_duration,
        state.total_loan_count,
        config.apy_alpha as u64,
    );

    let t_max = config.t_max;
    let apy_t = apy * t_max / SCALE;
    assert!(apy_t < SCALE, "APY × T_max >= 1, borrowing unsafe");

    let max_borrow = calculate_max_borrow(
        collateral_power,
        apy,
        t_max,
        config.fee_borrow_bps,
        config.safety_buffer_bps,
    );

    assert!(
        power_requested as u64 <= max_borrow,
        "Requested borrow exceeds maximum allowed"
    );

    let borrow_fee: u32 = (power_requested as u64 * config.fee_borrow_bps as u64 / 10000) as u32;
    let principal_net: u32 = power_requested.saturating_sub(borrow_fee);

    let calculated_reserve = calculate_reserve(principal_net as u64, apy, t_max);
    let min_reserve = (principal_net as u64 * config.min_reserve_bps as u64) / 10000;
    let reserve = calculated_reserve.max(min_reserve);

    assert!(reserve > 0, "Reserve must be positive to protect lenders");

    let buffer = (collateral_power as u64 * config.safety_buffer_bps as u64) / 10000;
    let total_required = principal_net as u64 + reserve + borrow_fee as u64 + buffer;
    assert!(
        total_required <= collateral_power as u64,
        "Insufficient collateral for borrow + reserve + fee + buffer"
    );

    // Check pool has enough liquidity
    assert!(
        state.total_offer >= principal_net as u64,
        "Insufficient liquidity in pool"
    );

    // Update pool
    state.total_offer -= principal_net as u64;
    state.total_borrowed_power += principal_net as u64;

    state.total_weight += reserve;

    // Lock card
    nft.locked_by_action = Action::Borrow;
    write_nft(&env, owner.clone(), token_id.clone(), nft);

    // Send fee to Hawaii pot (deduct from card POWER)
    let mut nft_for_fee = read_nft(&env.clone(), owner.clone(), token_id.clone()).unwrap();
    assert!(nft_for_fee.power >= borrow_fee, "Insufficient card power for fee");
    nft_for_fee.power = nft_for_fee.power.saturating_sub(borrow_fee);
    write_nft(&env, owner.clone(), token_id.clone(), nft_for_fee);

    accumulate_pot_internal(&env, 0, borrow_fee, 0, Some(owner.clone()), Some(Action::Borrow));
    emit_fee_to_hawaii(&env, borrow_fee, Action::Borrow);

    user.power += principal_net;
    write_user(&env.clone(), owner.clone(), user);

    let borrowing = Borrowing {
        borrower: owner.clone(),
        category: category.clone(),
        token_id: token_id.clone(),
        principal: principal_net,
        reserve,
        collateral_power,
        borrowed_at: env.ledger().timestamp(),
        last_liquidation_index: state.liquidation_index,
        weight: reserve,
    };

    write_borrowing(
        env.clone(),
        owner.clone(),
        category.clone(),
        token_id.clone(),
        borrowing,
    );

    write_state(&env, &state);

    emit_borrow(&env, &owner);
    emit_borrow_opened(&env, &owner, principal_net, reserve, collateral_power, borrow_fee);
    emit_card_locked(&env, &owner, Action::Borrow);
    emit_apy_updated(&env, state.total_demand, state.total_offer, state.total_loan_duration, config.apy_alpha as u64, apy);

    mint_terry(&env, owner.clone(), config.terry_per_borrow);
    emit_terry_awarded(&env, &owner, config.terry_per_borrow, Action::Borrow, config.terry_per_borrow);

    let mut balance = read_balance(&env);
    balance.haw_ai_terry += config.terry_per_borrow * config.haw_ai_percentage as i128 / 100;
    write_balance(&env, &balance);
}

fn apply_haircut(env: &Env, borrowing: &mut Borrowing, state: &State) -> (u64, bool) {
    let pending_haircut = if state.liquidation_index > borrowing.last_liquidation_index {
        let delta_l = state.liquidation_index - borrowing.last_liquidation_index;
        (delta_l * borrowing.weight) / SCALE
    } else {
        0
    };

    borrowing.last_liquidation_index = state.liquidation_index;

    let reserve_remaining = borrowing.reserve.saturating_sub(pending_haircut);
    borrowing.reserve = reserve_remaining;
    borrowing.weight = reserve_remaining;

    // Check if fully liquidated (collateral POWER = 0)
    let liquidated = reserve_remaining == 0;

    (pending_haircut, liquidated)
}

pub fn touch_loan(env: Env, borrower: Address, category: Category, token_id: TokenId) {
    let state = read_state(&env);
    let mut borrowing = read_borrowing(
        env.clone(),
        borrower.clone(),
        category.clone(),
        token_id.clone(),
    );

    let (haircut, liquidated) = apply_haircut(&env, &mut borrowing, &state);

    if liquidated {
        let mut nft = read_nft(&env, borrowing.borrower.clone(), borrowing.token_id.clone()).unwrap();
        nft.power = 0;
        nft.locked_by_action = Action::None;
        write_nft(&env, borrowing.borrower.clone(), borrowing.token_id.clone(), nft);

        remove_borrowing(env.clone(), borrower.clone(), category.clone(), token_id.clone());

        emit_loan_touched(&env, &borrower, haircut, 0, true);
        emit_card_unlocked(&env, &borrower);
    } else {
        write_borrowing(
            env.clone(),
            borrower.clone(),
            category.clone(),
            token_id.clone(),
            borrowing.clone(),
        );

        emit_loan_touched(&env, &borrower, haircut, borrowing.reserve, false);
    }
}

// Touch multiple loans in batch
pub fn touch_loans(env: Env, loan_ids: Vec<(Address, Category, TokenId)>) {
    for loan_id in loan_ids.iter() {
        let (borrower, category, token_id) = loan_id;
        touch_loan(env.clone(), borrower, category, token_id);
    }
}

pub fn repay(env: Env, user: Address, category: Category, token_id: TokenId) {
    user.require_auth();
    let mut user_data = read_user(&env, user);
    let owner = user_data.owner.clone();

    assert!(
        category == Category::Resource || category == Category::Leader,
        "Invalid Category to repay"
    );

    let mut nft = read_nft(&env.clone(), owner.clone(), token_id.clone()).unwrap();
    assert!(
        nft.locked_by_action == Action::Borrow,
        "Card is not locked by borrow action"
    );

    let mut state = read_state(&env);
    let mut borrowing = read_borrowing(
        env.clone(),
        owner.clone(),
        category.clone(),
        token_id.clone(),
    );

    let config = read_config(&env);
    let loan_duration = (env.ledger().timestamp() - borrowing.borrowed_at) / 3_600;

    state.total_demand += borrowing.principal as u64 * loan_duration;
    state.total_loan_duration += loan_duration;
    state.total_loan_count += 1;

    let apy = calculate_apy(
        state.total_demand,
        state.total_offer,
        state.total_loan_duration,
        state.total_loan_count,
        config.apy_alpha as u64,
    );

    let interest_amount = calculate_interest(borrowing.principal as u64, apy, loan_duration);
    let total_repay = borrowing.principal as u64 + interest_amount;

    assert!(
        user_data.power as u64 >= total_repay,
        "Insufficient Global POWER to repay"
    );

    // Apply haircut AFTER checking user can repay
    // Even if reserve is depleted (liquidated), allow repayment if user has sufficient power
    let (haircut, liquidated) = apply_haircut(&env, &mut borrowing, &state);

    // If liquidated but user can repay, process the repayment anyway
    // User loses collateral reserve but clears their debt

    user_data.power = (user_data.power as u64 - total_repay) as u32;
    write_user(&env, owner.clone(), user_data);

    // Return principal + interest to pool (borrower pays both)
    // Bug Fix: Was only returning principal, now returning total_repay (principal + interest)
    state.total_offer += total_repay; // Principal + interest returned to pool for lenders
    state.total_interest += interest_amount; // Track total interest collected
    state.total_borrowed_power -= borrowing.principal as u64;
    state.total_weight = state.total_weight.saturating_sub(borrowing.weight);

    write_state(&env, &state);

    nft.locked_by_action = Action::None;
    write_nft(&env, owner.clone(), token_id.clone(), nft);

    // Emit events
    emit_repay(&env, &owner);
    emit_loan_touched(&env, &owner, haircut, borrowing.reserve, false);
    emit_card_unlocked(&env, &owner);

    remove_borrowing(env.clone(), owner.clone(), category, token_id);

    // Mint terry to user as rewards
    mint_terry(&env, owner.clone(), config.terry_per_repay);
    emit_terry_awarded(&env, &owner, config.terry_per_repay, Action::Borrow, config.terry_per_repay);

    let mut balance = read_balance(&env);
    balance.haw_ai_terry += config.terry_per_repay * config.haw_ai_percentage as i128 / 100;
    write_balance(&env, &balance);
}

pub fn withdraw(env: Env, user: Address, category: Category, token_id: TokenId) {
    user.require_auth();
    let user_data = read_user(&env, user);
    let owner = user_data.owner.clone();

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
    let interest_due = calculate_interest(lending.principal_net as u64, apy, loan_duration);

    // SPEC REQUIREMENT: Check if interest reserve is sufficient before withdrawal
    // Per spec section 2.4: "Before payment, the contract checks whether the interest
    // reserve is sufficient to cover the total accrued interest. If the reserve is
    // insufficient, the contract triggers a liquidation process"

    // The total_weight represents the sum of all active borrower reserves
    // If interest needed exceeds available reserves, trigger proportional liquidation
    let total_interest_reserve = state.total_weight;

    // Check if we need to liquidate borrower reserves to cover interest
    if interest_due > 0 && total_interest_reserve < interest_due {
        let reserve_deficit = interest_due - total_interest_reserve;

        // Update liquidation index to trigger proportional haircuts on all borrowers
        if state.total_weight > 0 {
            let delta_l = (reserve_deficit * SCALE) / state.total_weight;
            state.liquidation_index += delta_l;

            emit_index_updated(&env, state.liquidation_index, delta_l, reserve_deficit, state.total_weight);
        }
    }

    // Now check pool liquidity for the actual payout
    let available_pool = state.total_offer;
    let total_payout = lending.principal_net as u64 + interest_due;

    if available_pool >= total_payout {
        // Sufficient liquidity in pool
        state.total_offer -= total_payout;
    } else {
        // Pool deficit: apply lazy pro-rata liquidation
        let payout_from_pool = available_pool;
        let pool_deficit = total_payout - payout_from_pool;

        state.total_offer = 0;

        // Update liquidation index for pool deficit as well
        if state.total_weight > 0 {
            let delta_l = (pool_deficit * SCALE) / state.total_weight;
            state.liquidation_index += delta_l;

            emit_index_updated(&env, state.liquidation_index, delta_l, pool_deficit, state.total_weight);
        }
    }

    write_state(&env, &state);

    // Return principal_net + interest to card
    nft.power += (lending.principal_net as u64 + interest_due) as u32;
    nft.locked_by_action = Action::None;
    write_nft(&env, owner.clone(), token_id.clone(), nft);

    emit_withdraw(&env, &owner);
    emit_withdraw_paid(&env, &owner, lending.principal_net, interest_due);
    emit_card_unlocked(&env, &owner);

    remove_lending(env.clone(), owner.clone(), category, token_id);

    // Mint terry to user as rewards
    mint_terry(&env, owner.clone(), config.terry_per_withdraw);
    emit_terry_awarded(&env, &owner, config.terry_per_withdraw, Action::Lend, config.terry_per_withdraw);

    let mut balance = read_balance(&env);
    balance.haw_ai_terry += config.terry_per_withdraw * config.haw_ai_percentage as i128 / 100;
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

// View function: get pending haircut for a loan
pub fn get_pending_haircut(env: Env, borrower: Address, category: Category, token_id: TokenId) -> u64 {
    let state = read_state(&env);
    let borrowing = read_borrowing(env.clone(), borrower, category, token_id);

    if state.liquidation_index > borrowing.last_liquidation_index {
        let delta_l = state.liquidation_index - borrowing.last_liquidation_index;
        (delta_l * borrowing.weight) / SCALE
    } else {
        0
    }
}
