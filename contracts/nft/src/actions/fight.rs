use crate::*;
use admin::{read_balance, read_config, write_balance};
use nft_info::{read_nft, write_nft, Action, Category};
use soroban_sdk::{contracttype, vec, Address, Env, IntoVal, Symbol, Val, Vec};
use storage_types::{DataKey, TokenId, BALANCE_BUMP_AMOUNT, BALANCE_LIFETIME_THRESHOLD};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Asset {
    Stellar(Address),
    Other(Symbol),
}

//price record definition
#[contracttype]
pub struct PriceData {
    price: i128,     //asset price at given point in time
    timestamp: u64   //recording timestamp
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SidePosition {
    Long,
    Short,
}

#[derive(Clone, PartialEq)]
#[contracttype]
pub enum Currency {
    BTC,
    ETH,
    XLM,
    SOL,
}

#[contracttype]
#[derive(Clone, PartialEq)]
pub struct Fight {
    pub owner: Address,
    pub category: Category,
    pub token_id: TokenId,
    pub currency: Currency,
    pub power: u32,
    pub trigger_price: i128,
    pub side_position: SidePosition,
    pub leverage: u32,
}

pub fn write_fight(env: Env, fee_payer: Address, category: Category, token_id: TokenId, fight: Fight) {
    fee_payer.require_auth();
    let owner = read_user_by_fee_payer(e, fee_payer).owner;

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
        fights.set(pos.try_into().unwrap(), fight)
    } else {
        fights.push_back(fight);
    }

    env.storage().persistent().set(&key, &fights);

    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

pub fn read_fight(env: Env, fee_payer: Address, category: Category, token_id: TokenId) -> Fight {
    let owner = read_user_by_fee_payer(e, fee_payer).owner;

    let key = DataKey::Fight(owner.clone(), category.clone(), token_id.clone());
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
    env.storage().persistent().get(&key).unwrap()
}

pub fn remove_fight(env: Env, fee_payer: Address, category: Category, token_id: TokenId) {

    let owner = read_user_by_fee_payer(e, fee_payer).owner;
    let key = DataKey::Fight(owner.clone(), category.clone(), token_id.clone());
    env.storage().persistent().remove(&key);

    if env.storage().persistent().has(&key) {
        env.storage().persistent().extend_ttl(
            &key,
            BALANCE_LIFETIME_THRESHOLD,
            BALANCE_BUMP_AMOUNT,
        );
    }

    let key = DataKey::Fights;
    let mut fights = read_fights(env.clone());
    if let Some(pos) = fights.iter().position(|fight| {
        fight.owner == owner && fight.category == category && fight.token_id == token_id
    }) {
        fights.remove(pos.try_into().unwrap());
    }

    env.storage().persistent().set(&key, &fights);

    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

pub fn read_fights(env: Env) -> Vec<Fight> {
    let key = DataKey::Fights;
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or(vec![&env.clone()])
}

pub fn get_currency_price(env: Env, oracle_contract_id: Address, currency: Currency) -> i128 {
    // let config = read_config(&env);
    let asset = match currency {
        Currency::BTC => Asset::Other(Symbol::new(&env, "BTC")),
        Currency::ETH => Asset::Other(Symbol::new(&env, "ETH")),
        Currency::XLM => Asset::Stellar(oracle_contract_id.clone()),
        Currency::SOL => Asset::Other(Symbol::new(&env, "SOL")),
    };
    let args: Vec<Val> = (asset.clone(),).into_val(&env);
    let function_symbol = Symbol::new(&env, "lastprice");

    let asset_price: Option<PriceData> = 
        env.invoke_contract(&oracle_contract_id, &function_symbol, args);

    if let Some( asset_price) = asset_price {
        asset_price.price
    } else {
        0
    }
}

pub fn open_position(
    env: Env,
    fee_payer: Address,
    category: Category,
    token_id: TokenId,
    currency: Currency,
    side_position: SidePosition,
    leverage: u32,
) {
    fee_payer.require_auth();
    let owner = read_user_by_fee_payer(e, fee_payer).owner;


    let mut nft = read_nft(&env, owner.clone(), category.clone(), token_id.clone());
    nft.locked_by_action = Action::Fight;

    let config = read_config(&env);
    let power_fee = config.power_action_fee * nft.power / 100;

    nft.power -= power_fee;

    let mut balance = read_balance(&env);
    balance.haw_ai_power += power_fee;
    write_balance(&env, &balance);

    write_nft(
        &env,
        owner.clone(),
        category.clone(),
        token_id.clone(),
        nft.clone(),
    );

    // get currency price from oracle
    let config = read_config(&env.clone());
    let trigger_price = get_currency_price(env.clone(), config.oracle_contract_id, currency.clone());

    write_fight(
        env.clone(),
        owner.clone(),
        category.clone(),
        token_id.clone(),
        Fight {
            owner,
            category,
            token_id,
            currency,
            power: nft.power,
            trigger_price,
            side_position,
            leverage,
        },
    );
}

pub fn close_position(env: Env, fee_payer: Address, category: Category, token_id: TokenId) {
    let owner = read_user_by_fee_payer(e, fee_payer).owner;
    let mut nft = read_nft(&env, owner.clone(), category.clone(), token_id.clone());
    nft.locked_by_action = Action::None;

    let fight = read_fight(
        env.clone(),
        owner.clone(),
        category.clone(),
        token_id.clone(),
    );

    let config = read_config(&env.clone());
    let current_price = get_currency_price(env.clone(), config.oracle_contract_id, fight.currency);
    let power = fight.power as i32
        * if fight.trigger_price == 0 {
            0
        } else {
            (if fight.side_position == SidePosition::Long {
                current_price - fight.trigger_price
            } else {
                fight.trigger_price - current_price
            } / fight.trigger_price) as i32
        }
        * fight.leverage as i32
        / 100;

    if power < 0 {
        if nft.power < -power as u32 {
            nft.power = 0;
        } else {
            nft.power -= power as u32;
        }
    } else {
        nft.power += power as u32;
    }

    write_nft(&env, owner.clone(), category.clone(), token_id.clone(), nft);

    remove_fight(
        env.clone(),
        owner.clone(),
        category.clone(),
        token_id.clone(),
    );
}
