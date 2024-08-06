use crate::admin::Config;
use crate::storage_types::Level;
use crate::{admin::{read_config, read_administrator}, storage_types::DataKey};
use soroban_sdk::{contracttype, token, Address, Env};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct User {
    pub power: u32,
    pub owner: Address,
    // pub borrowed_power: u32,
}

pub fn read_user(e: &Env, user: Address) -> User {
    let key = DataKey::User(user.clone());
    e.storage().persistent().get(&key).unwrap_or(User {
        power: 0,
        owner: user,
    })
}

pub fn write_user(e: &Env, user_info: User) {
    let key = DataKey::User(user_info.owner.clone());
    e.storage().persistent().set(&key, &user_info);
}

pub fn get_user_level(e: &Env, user: Address) -> u32 {
    let config: Config = read_config(&e.clone());
    let token = token::Client::new(&e.clone(), &config.terry_token.clone());
    let decimals = token.decimals();
    let balance: i128 = token.balance(&user);
    let balance = balance as u128 / 10u128.pow(decimals as u32);

    // Fetch the last level ID from storage
    let last_level_id = e
        .storage()
        .persistent()
        .get(&DataKey::LevelId)
        .unwrap_or(0u32);

    for i in 1..=last_level_id {
        let level: Level = e.storage().persistent().get(&DataKey::Level(i)).unwrap();
        if balance > level.minimum_terry && balance <= level.maximum_terry {
            return i;
        }
    }

    // Default level if no matching level is found
    1
}
