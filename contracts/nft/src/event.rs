use crate::storage_types::{PendingReward, PlayerReward, PotSnapshot};
use crate::nft_info::{Action};
use soroban_sdk::{Address, Env, BytesN, String, symbol_short};

// Event Emission
/// Emits an event when the pot is opened.
pub fn emit_pot_opened(env: &Env, round: u32, snapshot: &PotSnapshot) {
    env.events().publish(
        // (symbol_short!("pot_opened"), round),
        (symbol_short!("pot"), symbol_short!("open"), round),
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
        // (symbol_short!("share_calculated"), player.clone()),
        (symbol_short!("pot"), symbol_short!("share_cal"), player.clone()),
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
        // (symbol_short!("dogstar_fee_accumulated"),),
        (symbol_short!("pot"), symbol_short!("fee"), symbol_short!("accum")),
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
        // (symbol_short!("dogstar_fee_withdrawn"), recipient.clone()),
        (symbol_short!("pot"), symbol_short!("widthdraw"), recipient.clone()),
        (terry, power, xtar),
    );
}

/// Emits an event when the Dogstar fee percentage is updated.
pub fn emit_dogstar_fee_percentage_updated(env: &Env, old_fee: u32, new_fee: u32) {
    env.events()
        // .publish((symbol_short!("dogstar_fee_percentage_updated"),), (old_fee, new_fee));
        .publish((symbol_short!("pot"), symbol_short!("fee"), symbol_short!("change"),), (old_fee, new_fee));
}

/// Emits an event when a reward is claimed.
pub fn emit_reward_claimed(e: &Env, player: &Address, reward: &PendingReward) {
    e.events().publish(
        // (symbol_short!("reward_claimed"), player.clone(), reward.round_number),
        (symbol_short!("pot"), symbol_short!("reward"), symbol_short!("claimed"),),
        ( player.clone(), reward.round_number, reward.terry_amount, reward.power_amount, reward.xtar_amount),
    );
}

/// Emits an event when a reward is marked as pending due to missing trustline.
pub fn emit_reward_pending(e: &Env, player: &Address, reward: &PendingReward) {
    e.events().publish(
        (symbol_short!("pot"), symbol_short!("reward"), symbol_short!("pending"),),
        (player.clone(), reward.round_number, reward.terry_amount, reward.power_amount, reward.xtar_amount),
    );
}
