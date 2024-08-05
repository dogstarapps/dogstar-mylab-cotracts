use crate::storage_types::DataKey;
use crate::admin::Config;
use soroban_sdk::{contracttype, Address, Env, token};
use soroban_token_sdk::TokenUtils;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct User {
    pub power: u32,
    pub owner: Address,
    // pub borrowed_power: u32,
}

pub fn read_user(e: &Env, user: Address) -> User {
    let key = DataKey::User(user);
    e.storage().persistent().get(&key).unwrap_or(User {
        power:0, 
        owner: user
    })
}

pub fn read_user_by_fee_payer(e: &Env, user: Address) -> User {
    let key = DataKey::User(user);
    e.storage().persistent().get(&key).unwrap_or(User {
        power:0, 
        owner: user
    })
}


pub fn write_user(e: &Env, user: Address, user_info: User) {
    let key = DataKey::User(user);
    e.storage().persistent().set(&key, &user_info);
}

pub fn get_user_level(e: &Env, user: Address) -> u8 {

    let config: Config = read_config(&env);
    let token = token::Client::new(&env, &config.terry_token.clone());
    let balance : u128 = token.balance(&user); 


}