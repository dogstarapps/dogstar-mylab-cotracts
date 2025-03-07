use crate::admin::Config;

use crate::error::MyLabError;
use crate::storage_types::Level;
use crate::metadata::{read_metadata, CardMetadata };
use crate::nft_info::{
   read_nft, write_nft, Action, Card
};

use crate::{admin::{read_config, read_administrator}, storage_types::DataKey,  storage_types::TokenId, storage_types::BALANCE_BUMP_AMOUNT, storage_types::BALANCE_LIFETIME_THRESHOLD};
use soroban_sdk::{contracttype, token, Address, Env, Vec, storage, log};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct User {
    pub power: u32,
    pub owner: Address,
    // pub borrowed_power: u32,
}

pub fn add_card_to_owner(env: &Env, token_id: TokenId, user: Address )  -> Result<(),MyLabError>{
    log!(&env, "Add card to owner function");
    if let Some(card) = read_nft(&env, user.clone(), token_id.clone()) {
        log!(&env, "add_card_to_owner >> Found card {}", card.clone());
        let mut  user_card_ids =  read_owner_card(&env, user.clone());
        log!(&env, "add_card_to_owner >> User card ids {}", user_card_ids.clone());
        //let mut user_card_ids: Vec<u32> =
        //user_card_ids_key.get(&env).unwrap_or_else(|| Vec::new(&env));

        user_card_ids.push_back(token_id.clone());   
        write_owner_card(&env,user.clone(), user_card_ids);  
        //user_card_ids_key.set (&env, &user_card_ids);
       
        //get prototype card 
        // let  card_metadata = read_metadata(&env,token_id.clone().0);
        // let mut card  = Card {
        //     power : card_metadata.initial_power, 
        //     locked_by_action: Action::None,
        // };

        // write_nft(&env, user, token_id, card);
        Ok(()) 
    } else {
        log!(&env, "add_card_to_owner >> Card not found in add_card_to_owner");
        return  Err(MyLabError::NotNFT);
    }
       
}

pub fn read_user(e: &Env, user: Address) -> User {
    let key = DataKey::User(user.clone());
    e.storage().persistent().get(&key).unwrap_or(User {
        power: 0,
        owner: user,
    })
}

pub fn write_user(e: &Env, fee_payer:Address , user_info: User) {
    let key = DataKey::User(fee_payer);
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

pub fn write_owner_card(env: &Env,owner: Address, token_ids: Vec<TokenId>) {
    log!(&env, "write_owner_card >> Write owner card for {}, token_ids {}", owner.clone(), token_ids.clone());
    let key = DataKey::OwnerOwnedCardIds(owner);
    log!(&env, "write_owner_card >> Data key Owner Owned Ids {}", key);
    env.storage().persistent().set(&key, &token_ids);
    #[cfg(not(test))]
    {
        env.storage()
            .persistent()
            .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
    }
}

pub fn read_owner_card(env: &Env, owner: Address) -> Vec<TokenId> {
    log!(&env, "read_owner_card >> Read owner card for {}", owner.clone());
    let _owner = owner.clone();
    let key = DataKey::OwnerOwnedCardIds(owner);
    log!(&env, "read_owner_card >> Data key Owner owned ids {}, {}", _owner.clone(), key.clone());    #[cfg(not(test))] 
    
    if !env.storage().persistent().has(&key) {
        log!(&env, "No card found for owner {}", _owner);
        let empty_vec: Vec<TokenId> = Vec::new(&env);
        env.storage().persistent().set(&key, &empty_vec);
    }
    
    #[cfg(not(test))]
    {
        env.storage()
            .persistent()
            .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
    }
    
    log!(&env, "Card found for owner {}", _owner);
    env.storage().persistent().get(&key).unwrap_or_else(|| Vec::new(&env))
}
