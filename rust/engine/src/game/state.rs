use serde::{Deserialize, Serialize};

use crate::game::cards::{Card, ModifierCard};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    pub numbers: Vec<u8>,
    pub modifiers: Vec<ModifierCard>,
    pub has_lifeline: bool,
    pub status: PlayerStatus,
    pub total_score: i32,
}

/// The current state of a player within a round.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PlayerStatus {
    /// Still drawing cards.
    Active,
    /// Voluntarily stopped or frozen by another player.
    Stopped,
    /// Drew a duplicate number without a lifeline.
    Busted,
    /// Collected 7 unique number cards — round ends immediately.
    Flip7,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Deck {
    pub draw_pile: Vec<Card>,
    pub discard_pile: Vec<Card>,
}

/// An action-card effect that requires the current player to pick a target
/// before they can continue their turn.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PendingEffect {
    Freeze,
    Tap3,
}

/// Broad phase of the game used to gate valid actions.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GamePhase {
    /// A round is in progress.
    Playing,
    /// The round has ended; call `end_round` to tally scores and reset.
    RoundOver,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameState {
    pub players: Vec<Player>,
    pub deck: Deck,
    pub current_player: usize,
    /// The player whose actual turn it is.  Equal to `current_player` during
    /// normal play; differs only while a Tap3 victim is resolving chained
    /// effects they drew during their forced draws.  Used by
    /// `advance_after_effect` to know where to advance from once all chained
    /// effects are resolved.
    #[serde(default)]
    pub turn_holder: usize,
    pub round: u32,
    pub pending_effect: Option<PendingEffect>,
    /// Effects queued during a Tap3 forced-draw sequence (Freeze or Tap3 cards
    /// drawn by the victim).  Resolved one-by-one after all 3 forced draws
    /// complete, provided the victim did not bust.  Each entry is
    /// `(resolver_player_index, effect)`.  Auto-skipped if no active targets
    /// are available when the entry is popped.
    #[serde(default)]
    pub effect_queue: Vec<(usize, PendingEffect)>,
    pub phase: GamePhase,
}
