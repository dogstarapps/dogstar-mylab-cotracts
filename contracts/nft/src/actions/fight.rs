use crate::{nft_info::remove_nft, user_info::mint_terry, *};
use admin::{read_balance, read_config, write_balance};
use nft_info::{read_nft, write_nft, Action, Category};
use soroban_sdk::{contracttype, symbol_short, vec, Address, Env, IntoVal, Symbol, Val, Vec, log};
use storage_types::{DataKey, TokenId, BALANCE_BUMP_AMOUNT, BALANCE_LIFETIME_THRESHOLD};
use user_info::read_user;

use super::remove_owner_card;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Asset {
    Stellar(Address),
    Other(Symbol),
}

//price record definition
#[contracttype]
pub struct PriceData {
    price: i128,    //asset price at given point in time
    timestamp: u64, //recording timestamp
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SidePosition {
    Long,
    Short,
}

#[derive(Debug, Clone, PartialEq)]
#[contracttype]
pub enum FightCurrency {
    BTC,
    ETH,
    XLM,
    SOL,
}

#[contracttype]
#[derive(Debug, Clone, PartialEq)]
pub struct Fight {
    pub owner: Address,
    pub category: Category,
    pub token_id: TokenId,
    pub currency: FightCurrency,
    pub power: u32,
    pub trigger_price: i128,
    pub side_position: SidePosition,
    pub leverage: u32,
    pub amount_asset: i128,
}

pub fn write_fight(env: Env, user: Address, category: Category, token_id: TokenId, fight: Fight) {
    user.require_auth();
    let owner = read_user(&env, user).owner;

    let key = DataKey::Fight(owner.clone(), category.clone(), token_id.clone());
    env.storage().persistent().set(&key, &fight);
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);

    let key = DataKey::Fights;
    let mut fights = read_fights(env.clone());
    if let Some(pos) = fights.iter().position(|fight| {
        fight.owner == owner && fight.category == category && fight.token_id == token_id
    }) {
        fights.set(pos.try_into().unwrap(), fight.clone())
    } else {
        fights.push_back(fight.clone());
    }

    env.storage().persistent().set(&key, &fights);

    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);

    env.events().publish(
        (symbol_short!("fight"), symbol_short!("open")),
        fight.clone(),
    )
}

pub fn read_fight(env: Env, user: Address, category: Category, token_id: TokenId) -> Fight {
    let owner = read_user(&env, user).owner;

    let key = DataKey::Fight(owner.clone(), category.clone(), token_id.clone());
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
    env.storage().persistent().get(&key).unwrap()
}

pub fn remove_fight(env: Env, user: Address, category: Category, token_id: TokenId) {
    let owner = read_user(&env, user).owner;
    
    let key = DataKey::Fights;
    let mut fights = read_fights(env.clone());
    if let Some(pos) = fights.iter().position(|fight| {
        fight.owner == owner && fight.category == category && fight.token_id == token_id
    }) {
        let fight = read_fight(
            env.clone(),
            owner.clone(),
            category.clone(),
            token_id.clone(),
        );
        env.events().publish(
            (symbol_short!("fight"), symbol_short!("close")),
            fight.clone(),
        );
        fights.remove(pos.try_into().unwrap());
    }

    env.storage().persistent().set(&key, &fights);

    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);

    log!(&env, "remove_fight >> ", owner.clone());
    let key = DataKey::Fight(owner.clone(), category.clone(), token_id.clone());
    log!(&env, "remove_fight >> ", "key = ", key);
    env.storage().persistent().remove(&key);
}

pub fn read_fights(env: Env) -> Vec<Fight> {
    let key = DataKey::Fights;
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or(vec![&env.clone()])
}

pub fn get_currency_price(env: Env, oracle_contract_id: Address, currency: FightCurrency) -> i128 {
    // let config = read_config(&env);
    let asset = match currency {
        FightCurrency::BTC => Asset::Other(Symbol::new(&env, "BTC")),
        FightCurrency::ETH => Asset::Other(Symbol::new(&env, "ETH")),
        // FightCurrency::XLM => Asset::Other(oracle_contract_id.clone()),
        FightCurrency::XLM => Asset::Other(Symbol::new(&env, "XLM")),
        FightCurrency::SOL => Asset::Other(Symbol::new(&env, "SOL")),
    };
    let args: Vec<Val> = (asset.clone(),).into_val(&env);
    let function_symbol = Symbol::new(&env, "lastprice");

    let asset_price: Option<PriceData> =
        env.invoke_contract(&oracle_contract_id, &function_symbol, args);

    if let Some(asset_price) = asset_price {
        asset_price.price
    } else {
        0
    }
}

pub fn get_liquidation_price(fight: &Fight) -> i128 {
    match fight.side_position {
        SidePosition::Long => {
            fight.trigger_price * (1000000 - 1000000 / fight.leverage as i128) / 1000000
        }
        SidePosition::Short => {
            fight.trigger_price * (1000000 + 1000000 / fight.leverage as i128) / 1000000
        }
    }
}

pub fn check_liquidation(
    env: Env,
    liquidator: Address,
    user: Address,
    category: Category,
    token_id: TokenId,
) {
    // Require authorization from liquidator (can be anyone, but must be authenticated)
    liquidator.require_auth();
    
    let fight = read_fight(
        env.clone(),
        user.clone(),
        category.clone(),
        token_id.clone(),
    );
    let config = read_config(&env);
    let current_price = get_currency_price(
        env.clone(),
        config.oracle_contract_id,
        fight.currency.clone(),
    );
    
    // Validate oracle price is recent and reasonable
    assert!(current_price > 0, "Invalid oracle price");
    assert!(current_price < i128::MAX / 2, "Oracle price too high");
    
    let liq_price = get_liquidation_price(&fight);
    let is_liquidated = match fight.side_position {
        SidePosition::Long => current_price <= liq_price,
        SidePosition::Short => current_price >= liq_price,
    };
    
    // Only allow liquidation if position is actually underwater
    assert!(is_liquidated, "Position is not liquidatable");
    
    if is_liquidated {
        let mut nft = read_nft(&env, user.clone(), token_id.clone()).unwrap();
        nft.power = 0;
        remove_owner_card(&env, user.clone(), token_id.clone());
        remove_nft(&env, user.clone(), token_id.clone());
        remove_fight(env, user, category, token_id);
    }
}

pub fn open_position(
    env: Env,
    user: Address,
    category: Category,
    token_id: TokenId,
    currency: FightCurrency,
    side_position: SidePosition,
    leverage: u32,
    power_staked: u32,
) {
    user.require_auth();
    let owner = read_user(&env, user).owner;
    let mut nft = read_nft(&env, owner.clone(), token_id.clone()).unwrap();
    log!(&env, "fight >> nft to fight = ", nft);
    assert_eq!(nft.locked_by_action, Action::None, "Card is locked");
    assert!(leverage >= 1 && leverage <= 100, "Invalid leverage");
    assert!(power_staked > 0, "Power staked must be positive");
    let config = read_config(&env);

    // Deduct fee and staked POWER
    let power_fee = config.power_action_fee * power_staked / 100;
    assert!(nft.power >= power_staked + power_fee, "Insufficient POWER");
    nft.power = nft
        .power
        .checked_sub(power_staked + power_fee)
        .expect("Insufficient POWER");

    let mut balance = read_balance(&env);
    balance.haw_ai_power += power_fee;

    // Calculate position
    let power_to_usdc_rate = config.power_to_usdc_rate;
    let margin_usdc = (power_staked as i128) * power_to_usdc_rate / 10000;
    let position_size = margin_usdc * leverage as i128;

    // Get currency price from oracle (1)
    let mut trigger_price = 0;

    #[cfg(not(test))]
    {
        trigger_price =
            get_currency_price(env.clone(), config.oracle_contract_id, currency.clone());
    }
    log!(&env, "fight >> trigger_price = ", trigger_price);
    // #[cfg(test)]
    // {
    //     trigger_price = 8382580000; // Mock price for tests (83,825.8 USDC)

    // Get currency price from oracle (2)

    // let trigger_price = 8382580000; // Mock price for tests (83,825.8 USDC)

    // Enhanced oracle price validation
    assert!(trigger_price > 0, "Invalid oracle price: must be positive");
    assert!(trigger_price < i128::MAX / 100, "Oracle price exceeds maximum");
    
    // TODO: Add staleness check when timestamp is available from oracle
    // assert!(price_timestamp > env.ledger().timestamp() - 3600, "Oracle price too stale");

    let amount_asset = position_size
        .checked_mul(1000000)
        .expect("Overflow")
        / trigger_price;

    // Store fight
    nft.locked_by_action = Action::Fight;
    write_nft(&env, owner.clone(), token_id.clone(), nft);
    write_fight(
        env.clone(),
        owner.clone(),
        category.clone(),
        token_id.clone(),
        Fight {
            owner: owner.clone(),
            category,
            token_id,
            currency,
            power: power_staked,
            trigger_price,
            side_position,
            leverage,
            amount_asset,
        },
    );

    // Mint TERRY rewards
    mint_terry(&env, owner, config.terry_per_fight);
    balance.haw_ai_terry += config.terry_per_fight * config.haw_ai_percentage as i128 / 100;
    write_balance(&env, &balance);
}

pub fn close_position(env: Env, user: Address, category: Category, token_id: TokenId) {
    user.require_auth();
    let owner = read_user(&env, user.clone()).owner;
    let mut nft = read_nft(&env, owner.clone(), token_id.clone()).unwrap();
    log!(&env, "read nft = ", nft.clone());
    let fight = read_fight(
        env.clone(),
        owner.clone(),
        category.clone(),
        token_id.clone(),
    );
    log!(&env, "read fight = ", fight.clone());
    let config = read_config(&env);
    let mut balance = read_balance(&env);

    // Deduct fee
    // let power_fee = config.power_action_fee * fight.power / 100;
    // assert!(nft.power >= power_fee, "Insufficient POWER for fee");
    // nft.power -= power_fee;
    // balance.haw_ai_power += power_fee;
    // log!(&env, "calculated power fee = ", power_fee.clone());
    // Calculate PnL
    let power_to_usdc_rate = config.power_to_usdc_rate;
    let margin_usdc = (fight.power as i128) * power_to_usdc_rate / 10000;
    let position_size = margin_usdc * fight.leverage as i128;
    let mut current_price = 0;
    #[cfg(not(test))]
    {
        current_price = get_currency_price(env.clone(), config.oracle_contract_id, fight.currency);
    }
    #[cfg(test)]
    {
        current_price = 86000; // Mock price for tests (86,000 USDC)
    }
    log!(&env, "current asset price", current_price.clone());
    assert!(current_price > 0, "Invalid oracle price");
    assert!(fight.trigger_price > 0, "Invalid trigger price");
    let pnl_usdc = position_size * (current_price - fight.trigger_price) / fight.trigger_price;
    let pnl_usdc = if fight.side_position == SidePosition::Long { pnl_usdc } else { -1 * pnl_usdc };
    let pnl_power = pnl_usdc * 10000 / power_to_usdc_rate;
    log!(&env, "pnl = ", pnl_usdc, pnl_power);

    let card_metadata = crate::metadata::read_metadata(&env, token_id.0);

    // Calculate trading result: staked fight power + P&L
    let trading_result = fight.power as i128 + pnl_power;
    log!(&env, "trading calculation: fight.power =", fight.power, "pnl_power =", pnl_power, "trading_result =", trading_result);

    let final_power = if trading_result < 0 {
        nft.power
    } else {
        nft.power + trading_result as u32
    };

    log!(&env, "power calculation: nft.power =", nft.power, "final_power =", final_power);

    if final_power == 0 {
        remove_owner_card(&env, user.clone(), token_id.clone());
        remove_nft(&env, user.clone(), token_id.clone());
    } else {
        nft.power = final_power.min(card_metadata.max_power);
        nft.locked_by_action = Action::None;
        write_nft(&env, owner.clone(), token_id.clone(), nft);
    }
    log!(&env, "remove fight", fight.token_id.clone());
    // Remove fight
    remove_fight(env.clone(), owner.clone(), category.clone(), token_id);

    // Mint TERRY rewards
    mint_terry(&env, owner, config.terry_per_fight);
    balance.haw_ai_terry += config.terry_per_fight * config.haw_ai_percentage as i128 / 100;
    write_balance(&env, &balance);
}
