use crate::*;
use admin::{mint_terry, read_balance, read_config, write_balance};
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
    pub staked_block: u32,
}

pub fn write_stake(env: &Env, fee_payer: Address, category: Category, token_id: TokenId, stake: Stake) {
    let owner = read_user(&env, fee_payer).owner;
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

pub fn remove_stake(env: &Env, fee_payer: Address, category: Category, token_id: TokenId) {
    let owner = read_user(&env, fee_payer).owner;

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

    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

pub fn read_stake(env: &Env, fee_payer: Address, category: Category, token_id: TokenId) -> Stake {
    let owner = read_user(&env, fee_payer).owner;
    let key = DataKey::Stake(owner, category, token_id);
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);

    env.storage().persistent().get(&key).unwrap()
}

pub fn stake(
    env: Env,
    fee_payer: Address,
    category: Category,
    token_id: TokenId,
    stake_power: u32,
    period_index: u32,
) {
    fee_payer.require_auth();
    assert!(
        category == Category::Skill || category == Category::Leader,
        "Invalid Category to stake"
    );
    let owner = read_user(&env, fee_payer).owner;

    let config = read_config(&env);
    let power_fee = config.power_action_fee * stake_power / 100;

    let mut nft = read_nft(&env, owner.clone(), category.clone(), token_id.clone());
    assert!(nft.locked_by_action == Action::None, "Locked NFT");

    nft.locked_by_action = Action::Stake;
    nft.power -= stake_power;

    let mut balance = read_balance(&env);
    balance.haw_ai_power += power_fee;
    write_balance(&env, &balance);

    write_nft(&env, owner.clone(), category.clone(), token_id.clone(), nft);

    write_stake(
        &env,
        owner.clone(),
        category.clone(),
        token_id.clone(),
        Stake {
            owner,
            category,
            token_id,
            power: stake_power - power_fee,
            period: config.stake_periods.get(period_index).unwrap(),
            interest_percentage: config.stake_interest_percentages.get(period_index).unwrap(),
            staked_block: env.ledger().sequence(),
        },
    )
}

pub fn increase_stake_power(
    env: Env,
    fee_payer: Address,
    category: Category,
    token_id: TokenId,
    increase_power: u32,
) {
    fee_payer.require_auth();
    let owner = read_user(&env, fee_payer).owner;

    let mut nft = read_nft(&env, owner.clone(), category.clone(), token_id.clone());
    assert!(nft.locked_by_action == Action::Stake, "Can't find staked");

    let mut stake = read_stake(&env, owner.clone(), category.clone(), token_id.clone());
    stake.power += increase_power;

    let config = read_config(&env);
    let power_fee = config.power_action_fee * increase_power / 100;
    stake.power -= power_fee;

    nft.power -= increase_power;
    write_nft(&env, owner.clone(), category.clone(), token_id.clone(), nft);

    let mut balance = read_balance(&env);
    balance.haw_ai_power += power_fee;
    write_balance(&env, &balance);

    write_stake(
        &env,
        owner.clone(),
        category.clone(),
        token_id.clone(),
        stake,
    );
}

pub fn unstake(env: Env, fee_payer: Address, category: Category, token_id: TokenId) {
    fee_payer.require_auth();
    let owner = read_user(&env, fee_payer).owner;
    let mut nft = read_nft(&env, owner.clone(), category.clone(), token_id.clone());
    assert!(nft.locked_by_action == Action::Stake, "Can't find staked");

    let current_block = env.ledger().sequence();

    let stake = read_stake(&env, owner.clone(), category.clone(), token_id.clone());
    assert!(
        stake.staked_block + stake.period <= current_block,
        "Locked Period"
    );

    let staked_time = current_block - stake.staked_block;
    let interest_amount = if stake.period == 0 {
        0
    } else {
        stake.interest_percentage * stake.power * staked_time / stake.period / 100
    };
    nft.power += interest_amount;

    write_nft(&env, owner.clone(), category.clone(), token_id.clone(), nft);

    let config = read_config(&env);
    let terry_amount = config.terry_per_power * interest_amount as i128;

    mint_terry(&env, owner.clone(), terry_amount);

    remove_stake(&env, owner.clone(), category.clone(), token_id.clone());
}
