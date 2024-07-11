use crate::{storage_types::DataKey};
use soroban_sdk::{contracttype, Address, Env};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct User {
    pub level: u32,
    // pub borrowed_power: u32,
}

pub fn read_user(e: &Env, user: Address) -> User {
    let key = DataKey::User(user);
    e.storage().persistent().get(&key).unwrap_or(User {
        level: 0,
        // borrowed_power: 0
    })
}

pub fn write_user(e: &Env, user: Address, user_info: User) {
    let key = DataKey::User(user);
    e.storage().persistent().set(&key, &user_info);
}