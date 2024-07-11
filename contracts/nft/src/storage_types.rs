use soroban_sdk::{contracttype, Address};

use crate::nft_info::Category;

pub(crate) const DAY_IN_LEDGERS: u32 = 17280;
pub(crate) const INSTANCE_BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
pub(crate) const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

pub(crate) const BALANCE_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
pub(crate) const BALANCE_LIFETIME_THRESHOLD: u32 = BALANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

#[derive(Clone, PartialEq)]
#[contracttype]
pub struct TokenId(pub u32);

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Config,
    Balance,
    Whitelist(Address),
    User(Address),
    Card(Address, Category, TokenId),
    Stakes,
    Stake(Address, Category, TokenId),
    Decks,
    Deck(Address),
    Lendings,
    Lending(Address, Category, TokenId),
    Fights,
    Fight(Address, Category, TokenId),
}