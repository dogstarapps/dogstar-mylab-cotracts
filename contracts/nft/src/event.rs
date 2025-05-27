use crate::storage_types::{PendingReward, PlayerReward, PotSnapshot};
use soroban_sdk::{Address, Env};

// Event Emission
/// Emits an event when the pot is opened.
pub fn emit_pot_opened(env: &Env, round: u32, snapshot: &PotSnapshot) {
    env.events().publish(
        ("pot_opened", round),
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
        ("share_calculated", player.clone()),
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
) {
    env.events().publish(
        ("dogstar_fee_accumulated",),
        (terry, power, xtar, fee_percentage),
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
        ("dogstar_fee_withdrawn", recipient.clone()),
        (terry, power, xtar),
    );
}

/// Emits an event when the Dogstar fee percentage is updated.
pub fn emit_dogstar_fee_percentage_updated(env: &Env, old_fee: u32, new_fee: u32) {
    env.events()
        .publish(("dogstar_fee_percentage_updated",), (old_fee, new_fee));
}

/// Emits an event when a reward is claimed.
pub fn emit_reward_claimed(e: &Env, player: &Address, reward: &PendingReward) {
    e.events().publish(
        ("reward_claimed", player.clone(), reward.round_number),
        (reward.terry_amount, reward.power_amount, reward.xtar_amount),
    );
}

/// Emits an event when a reward is marked as pending due to missing trustline.
pub fn emit_reward_pending(e: &Env, player: &Address, reward: &PendingReward) {
    e.events().publish(
        ("reward_pending", player.clone(), reward.round_number),
        (reward.terry_amount, reward.power_amount, reward.xtar_amount),
    );
}
