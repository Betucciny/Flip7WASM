use serde::{Deserialize, Serialize};

use crate::game::cards::Card;

/// All legal actions a player can take on their turn.
///
/// There are two draw variants:
/// - [`Action::Draw`] — **simulation**: the engine pops a random card from its
///   internal shuffled draw pile.
/// - [`Action::DrawKnown`] — **real-game tracking**: supply the exact card
///   that was flipped. The engine removes it from the deck model so the
///   remaining draw pile always reflects the true set of unseen cards.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Action {
    /// Draw the top card of the (internally shuffled) draw pile.
    Draw,
    /// Record a specific card drawn in the real game.
    /// JS shape: `{ "type": "DrawKnown", "card": { "type": "Number", "value": 7 } }`
    DrawKnown { card: Card },
    /// End this player's turn and lock in their current score.
    Stop,
    /// Resolve a pending Freeze effect by choosing which player to freeze.
    /// JS shape: `{ "type": "Freeze", "target": 2 }`
    Freeze { target: usize },
    /// Resolve a pending Tap3 effect by choosing which player draws 3 cards.
    /// JS shape: `{ "type": "Tap3", "target": 1 }`
    Tap3 { target: usize },
    /// Real-game tracking variant of Tap3: supply the exact cards the target drew
    /// instead of letting the engine draw randomly. This keeps the deck model
    /// accurate for future probability calculations.
    /// JS shape: `{ "type": "Tap3Known", "target": 1, "cards": [{...}, ...] }`
    Tap3Known { target: usize, cards: Vec<Card> },
}
