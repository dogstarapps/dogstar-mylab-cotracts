use crate::storage_types::{PendingReward, PlayerReward, PotSnapshot};
use crate::nft_info::{Action};
use soroban_sdk::{Address, Env, BytesN, String, symbol_short};

// Event Emission
/// Emits an event when the pot is opened.
pub fn emit_pot_opened(env: &Env, round: u32, snapshot: &PotSnapshot) {
    env.events().publish(
        (symbol_short!("pot_open"), round),
        (
            snapshot.total_terry,
            snapshot.total_power,
            snapshot.total_xtar,
            snapshot.total_participants,
            snapshot.total_effective_power,
        ),
    );
}

/// Emits an event when a player's share is calculated.
pub fn emit_share_calculated(env: &Env, player: &Address, reward: &PlayerReward) {
    env.events().publish(
        (symbol_short!("share_cal"), player.clone()),
        (
            reward.round_number,
            reward.share_percentage,
            reward.effective_power,
            reward.deck_bonus,
            reward.deck_categories,
        ),
    );
}

/// Emits an event when Dogstar fees are accumulated.
pub fn emit_dogstar_fee_accumulated(
    env: &Env,
    terry: i128,
    power: u32,
    xtar: i128,
    fee_percentage: u32,
    from: Option<Address>,
    action: Option<Action>
) {
    let action_val = action.unwrap_or(Action::None);
    
    // let demo_account_strkey = "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF";
    
    // Convert &str to soroban_sdk::String
    // let demo_address_string = String::from_str(&env, demo_account_strkey); 
    
    // Use the soroban_sdk::String to create the Address
    // let demo_address = Address::from_string(&demo_address_string);
    // let from_val = from.unwrap_or(Address::from_string(String::from(&"GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF")));
    
    env.events().publish(
        (symbol_short!("fee_acc"),),
        (terry, power, xtar, fee_percentage, from, action_val),
    );
}

/// Emits an event when Dogstar fees are withdrawn.
pub fn emit_dogstar_fee_withdrawn(
    env: &Env,
    recipient: &Address,
    terry: i128,
    power: u32,
    xtar: i128,
) {
    env.events().publish(
        (symbol_short!("fee_wd"), recipient.clone()),
        (terry, power, xtar),
    );
}

/// Emits an event when the Dogstar fee percentage is updated.
pub fn emit_dogstar_fee_percentage_updated(env: &Env, old_fee: u32, new_fee: u32) {
    env.events()
        .publish((symbol_short!("fee_pct"),), (old_fee, new_fee));
}

/// Emits an event when a reward is claimed.
pub fn emit_reward_claimed(e: &Env, player: &Address, reward: &PendingReward) {
    e.events().publish(
        (symbol_short!("rwd_cld"), player.clone(), reward.round_number),
        (reward.terry_amount, reward.power_amount, reward.xtar_amount),
    );
}

/// Emits an event when a reward is marked as pending due to missing trustline.
pub fn emit_reward_pending(e: &Env, player: &Address, reward: &PendingReward) {
    e.events().publish(
        (symbol_short!("reward_pd"), player.clone(), reward.round_number),
        (reward.terry_amount, reward.power_amount, reward.xtar_amount),
    );
}

/// Emits an event when rewards are claimed from HAW AI pot.
pub fn emit_rewards_claimed(e: &Env, player: &Address, terry: i128, power: u32, xtar: i128) {
    e.events().publish(
        (symbol_short!("rwd_claim"), player.clone()),
        (terry, power, xtar),
    );
}

/// Emits an event when a card is burned.
pub fn emit_burn(env: &Env, player: &Address) {
    env.events().publish(
        (symbol_short!("burn"), player.clone()),
        (),
    );
}

/// Emits an event when a card is staked.
pub fn emit_stake(env: &Env, player: &Address) {
    env.events().publish(
        (symbol_short!("stake"), symbol_short!("open"), player.clone()),
        (),
    );
}

/// Emits an event when stake power is increased.
pub fn emit_stake_increased(env: &Env, player: &Address) {
    env.events().publish(
        (symbol_short!("stake"), symbol_short!("increase"), player.clone()),
        (),
    );
}

/// Emits an event when a card is unstaked.
pub fn emit_unstake(env: &Env, player: &Address) {
    env.events().publish(
        (symbol_short!("stake"), symbol_short!("close"), player.clone()),
        (),
    );
}

/// Emits an event when a card is lent.
pub fn emit_lend(env: &Env, player: &Address) {
    env.events().publish(
        (symbol_short!("lend"), symbol_short!("open"), player.clone()),
        (),
    );
}

/// Emits an event when lending is withdrawn.
pub fn emit_withdraw(env: &Env, player: &Address) {
    env.events().publish(
        (symbol_short!("lend"), symbol_short!("close"), player.clone()),
        (),
    );
}

/// Emits an event when borrowing is made.
pub fn emit_borrow(env: &Env, player: &Address) {
    env.events().publish(
        (symbol_short!("borrow"), symbol_short!("open"), player.clone()),
        (),
    );
}

/// Emits an event when repayment is made.
pub fn emit_repay(env: &Env, player: &Address) {
    env.events().publish(
        (symbol_short!("borrow"), symbol_short!("close"), player.clone()),
        (),
    );
}

/// Emits an event when a card is placed in a deck.
pub fn emit_deck_place(env: &Env, player: &Address) {
    env.events().publish(
        (symbol_short!("deck"), symbol_short!("place"), player.clone()),
        (),
    );
}

/// Emits an event when a card is replaced in a deck.
pub fn emit_deck_replace(env: &Env, player: &Address) {
    env.events().publish(
        (symbol_short!("deck"), symbol_short!("replace"), player.clone()),
        (),
    );
}

/// Emits an event when a card is removed from a deck.
pub fn emit_deck_remove(env: &Env, player: &Address) {
    env.events().publish(
        (symbol_short!("deck"), symbol_short!("remove"), player.clone()),
        (),
    );
}

/// Emits an event when a deck is completed (4 cards).
pub fn emit_deck_completed(env: &Env, player: &Address) {
    env.events().publish(
        (symbol_short!("deck"), symbol_short!("complete"), player.clone()),
        (),
    );
}

/// Emits an event when a card is minted.
pub fn emit_mint(env: &Env, player: &Address) {
    env.events().publish(
        (symbol_short!("mint"), player.clone()),
        (),
    );
}

/// Emits an event when a card is transferred.
pub fn emit_transfer(env: &Env, from: &Address, to: &Address) {
    env.events().publish(
        (symbol_short!("transfer"), from.clone(), to.clone()),
        (),
    );
}

/// Emits an event when APY is updated.
pub fn emit_apy_updated(env: &Env, demand: u64, supply: u64, duration: u64, alpha: u64, apy: u64) {
    env.events().publish(
        (symbol_short!("apy_upd"),),
        (demand, supply, duration, alpha, apy),
    );
}

/// Emits an event when lend is deposited with details.
pub fn emit_lend_deposited(env: &Env, lender: &Address, principal_gross: u32, fee: u32, principal_net: u32) {
    env.events().publish(
        (symbol_short!("lend_dep"), lender.clone()),
        (principal_gross, fee, principal_net),
    );
}

/// Emits an event when borrow is opened with reserve details.
pub fn emit_borrow_opened(env: &Env, borrower: &Address, principal: u32, reserve: u64, collateral: u32, fee: u32) {
    env.events().publish(
        (symbol_short!("bor_open"), borrower.clone()),
        (principal, reserve, collateral, fee),
    );
}

/// Emits an event when withdraw is paid.
pub fn emit_withdraw_paid(env: &Env, lender: &Address, principal: u32, interest: u64) {
    env.events().publish(
        (symbol_short!("wd_paid"), lender.clone()),
        (principal, interest),
    );
}

/// Emits an event when liquidation index is updated.
pub fn emit_index_updated(env: &Env, new_index: u64, delta_index: u64, deficit: u64, total_weight: u64) {
    env.events().publish(
        (symbol_short!("idx_upd"),),
        (new_index, delta_index, deficit, total_weight),
    );
}

/// Emits an event when a loan is touched (haircut applied).
pub fn emit_loan_touched(env: &Env, borrower: &Address, haircut: u64, reserve_remaining: u64, liquidated: bool) {
    env.events().publish(
        (symbol_short!("loan_tch"), borrower.clone()),
        (haircut, reserve_remaining, liquidated),
    );
}

/// Emits an event when card is locked.
pub fn emit_card_locked(env: &Env, owner: &Address, action: Action) {
    env.events().publish(
        (symbol_short!("card_lck"), owner.clone()),
        action,
    );
}

/// Emits an event when card is unlocked.
pub fn emit_card_unlocked(env: &Env, owner: &Address) {
    env.events().publish(
        (symbol_short!("card_ulk"), owner.clone()),
        (),
    );
}

/// Emits an event when fee is sent to Hawaii pot.
pub fn emit_fee_to_hawaii(env: &Env, amount: u32, action: Action) {
    env.events().publish(
        (symbol_short!("fee_haw"),),
        (amount, action),
    );
}

/// Emits an event when Terry is awarded.
pub fn emit_terry_awarded(env: &Env, player: &Address, amount: i128, action: Action, daily_total: i128) {
    env.events().publish(
        (symbol_short!("ter_awd"), player.clone()),
        (amount, action, daily_total),
    );
}
