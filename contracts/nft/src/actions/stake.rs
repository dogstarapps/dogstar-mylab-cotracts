use crate::{user_info::mint_terry, *};
use admin::{read_balance, read_config, read_state, write_balance, write_state};
use nft_info::{read_nft, write_nft, Action, Category};
use soroban_sdk::{contracttype, vec, Address, Env, Vec};
use storage_types::{DataKey, TokenId, BALANCE_BUMP_AMOUNT, BALANCE_LIFETIME_THRESHOLD};
use user_info::read_user;

#[contracttype]
#[derive(Clone, PartialEq)]
pub struct Stake {
    pub owner: Address,
    pub category: Category,
    pub token_id: TokenId,
    pub power: u32,
    pub period: u32,
    pub interest_percentage: u32,
    pub staked_time: u32,
}

pub fn write_stake(env: &Env, user: Address, category: Category, token_id: TokenId, stake: Stake) {
    let owner = read_user(&env, user).owner;
    let key = DataKey::Stake(owner.clone(), category.clone(), token_id.clone());
    env.storage().persistent().set(&key, &stake);
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);

    let key = DataKey::Stakes;
    let mut stakes = read_stakes(env.clone());
    if let Some(pos) = stakes.iter().position(|stake| {
        stake.owner == owner && stake.category == category && stake.token_id == token_id
    }) {
        stakes.set(pos.try_into().unwrap(), stake)
    } else {
        stakes.push_back(stake)
    }

    env.storage().persistent().set(&key, &stakes);

    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

pub fn read_stakes(env: Env) -> Vec<Stake> {
    let key = DataKey::Stakes;
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or(vec![&env.clone()])
}

pub fn remove_stake(env: &Env, user: Address, category: Category, token_id: TokenId) {
    let owner = read_user(&env, user).owner;

    let key = DataKey::Stake(owner.clone(), category.clone(), token_id.clone());
    env.storage().persistent().remove(&key);
    if env.storage().persistent().has(&key) {
        env.storage().persistent().extend_ttl(
            &key,
            BALANCE_LIFETIME_THRESHOLD,
            BALANCE_BUMP_AMOUNT,
        );
    }

    let key = DataKey::Stakes;
    let mut stakes = read_stakes(env.clone());
    if let Some(pos) = stakes.iter().position(|stake| {
        stake.owner == owner && stake.category == category && stake.token_id == token_id
    }) {
        stakes.remove(pos.try_into().unwrap());
    }

    env.storage().persistent().set(&key, &stakes);

    #[cfg(not(test))]
    {
        env.storage().persistent().extend_ttl(
            &key,
            BALANCE_LIFETIME_THRESHOLD,
            BALANCE_BUMP_AMOUNT,
        );
    }
}

pub fn read_stake(env: &Env, user: Address, category: Category, token_id: TokenId) -> Stake {
    let owner = read_user(&env, user).owner;
    let key = DataKey::Stake(owner, category, token_id);
    #[cfg(not(test))]
    {
        env.storage().persistent().extend_ttl(
            &key,
            BALANCE_LIFETIME_THRESHOLD,
            BALANCE_BUMP_AMOUNT,
        );
    }
    env.storage().persistent().get(&key).unwrap()
}

pub fn stake(env: Env, user: Address, category: Category, token_id: TokenId, period_index: u32) {
    user.require_auth();
    assert!(
        category == Category::Skill || category == Category::Leader,
        "Invalid Category to stake"
    );
    let owner = read_user(&env, user).owner;

    let mut nft = read_nft(&env, owner.clone(), token_id.clone()).unwrap();
    assert!(nft.locked_by_action == Action::None, "Locked NFT");

    let config = read_config(&env);
    // Validate period index bounds to avoid panic
    assert!(period_index < config.stake_periods.len(), "Invalid period index");
    assert!(period_index < config.stake_interest_percentages.len(), "Invalid period index");
    let power_fee = config.power_action_fee * nft.power / 100;

    nft.locked_by_action = Action::Stake;
    let staked_power = nft
        .power
        .checked_sub(power_fee)
        .expect("Insufficient POWER for fee");
    nft.power = 0;

    let mut balance = read_balance(&env);
    balance.haw_ai_power += power_fee;
    write_balance(&env, &balance);

    write_nft(&env, owner.clone(), token_id.clone(), nft);

    write_stake(
        &env,
        owner.clone(),
        category.clone(),
        token_id.clone(),
        Stake {
            owner,
            category,
            token_id,
            power: staked_power,
            period: config.stake_periods.get(period_index).unwrap(),
            interest_percentage: config.stake_interest_percentages.get(period_index).unwrap(),
            staked_time: env
                .ledger()
                .timestamp()
                .try_into()
                .expect("Timestamp exceeds u32 limit"),
        },
    );

    let mut state = read_state(&env);
    state.total_staked_power += staked_power as u64;

    write_state(&env, &state);
}

pub fn increase_stake_power(
    env: Env,
    user: Address,
    category: Category,
    token_id: TokenId,
    increase_power: u32,
) {
    user.require_auth();
    let owner = read_user(&env, user).owner;
    
    // Input validation
    assert!(increase_power > 0, "Increase power must be positive");
    assert!(increase_power <= u32::MAX / 2, "Increase power too large");

    let mut nft = read_nft(&env, owner.clone(), token_id.clone()).unwrap();
    assert!(nft.locked_by_action == Action::Stake, "Can't find staked");
    assert!(nft.power >= increase_power, "Insufficient NFT power");

    let mut stake = read_stake(&env, owner.clone(), category.clone(), token_id.clone());
    
    // Safe addition to prevent overflow
    stake.power = stake.power.checked_add(increase_power)
        .expect("Stake power overflow");

    let config = read_config(&env);
    let power_fee = config.power_action_fee.checked_mul(increase_power)
        .and_then(|v| v.checked_div(100))
        .expect("Fee calculation overflow");
    
    // Safe subtraction to prevent underflow
    stake.power = stake.power.checked_sub(power_fee)
        .expect("Insufficient stake power for fee");

    // Safe subtraction to prevent underflow
    nft.power = nft.power.checked_sub(increase_power)
        .expect("Insufficient NFT power");
    write_nft(&env, owner.clone(), token_id.clone(), nft);

    let mut balance = read_balance(&env);
    balance.haw_ai_power += power_fee;

    write_stake(
        &env,
        owner.clone(),
        category.clone(),
        token_id.clone(),
        stake,
    );

    // Mint terry to user as rewards
    mint_terry(&env, owner, config.terry_per_stake);

    balance.haw_ai_terry += config.terry_per_stake * config.haw_ai_percentage as i128 / 100;
    write_balance(&env, &balance);
}

pub fn unstake(env: Env, user: Address, category: Category, token_id: TokenId) {
    user.require_auth();
    let owner = read_user(&env, user).owner;
    let mut nft = read_nft(&env, owner.clone(), token_id.clone()).unwrap();
    assert!(nft.locked_by_action == Action::Stake, "Can't find staked");

    let current_time: u32 = env
        .ledger()
        .timestamp()
        .try_into()
        .expect("Timestamp exceeds u32 limit");

    let stake = read_stake(&env, owner.clone(), category.clone(), token_id.clone());
    #[cfg(not(test))]
    {
        assert!(
            stake.staked_time + stake.period <= current_time,
            "Locked Period"
        );
    }

    let interest_amount = stake.power * stake.interest_percentage / 100;
    nft.power += stake.power + interest_amount;
    nft.locked_by_action = Action::None;

    write_nft(&env, owner.clone(), token_id.clone(), nft);

    let config = read_config(&env);
    let terry_amount = config.terry_per_power * interest_amount as i128;

    mint_terry(&env, owner.clone(), terry_amount);

    remove_stake(&env, owner.clone(), category.clone(), token_id.clone());

    // Mint terry to user as rewards
    mint_terry(&env, owner, config.terry_per_stake);

    let mut balance = read_balance(&env);
    balance.haw_ai_terry += config.terry_per_stake * config.haw_ai_percentage as i128 / 100;
    write_balance(&env, &balance);
}
