use crate::game::cards::{ActionCard, Card};
use crate::game::deck::{create_deck, shuffle_deck};
use crate::game::scoring::calculate_score;
use crate::game::state::{Deck, GamePhase, GameState, Player, PlayerStatus};

// ── Game setup ───────────────────────────────────────────────────────────────

pub fn new_game(player_count: usize) -> GameState {
    let mut deck = create_deck();
    shuffle_deck(&mut deck);
    GameState {
        players: (0..player_count)
            .map(|_| Player {
                numbers: vec![],
                modifiers: vec![],
                has_lifeline: false,
                status: PlayerStatus::Active,
                total_score: 0,
            })
            .collect(),
        deck: Deck {
            draw_pile: deck,
            discard_pile: vec![],
        },
        current_player: 0,
        turn_holder: 0,
        round: 1,
        pending_effect: None,
        effect_queue: vec![],
        phase: GamePhase::Playing,
    }
}

// ── Round lifecycle ───────────────────────────────────────────────────────────

/// Returns `true` when the round is over:
/// - A player reached Flip7 (7 unique number cards), or
/// - All players are no longer `Active` (stopped or busted).
pub fn is_round_over(state: &GameState) -> bool {
    state
        .players
        .iter()
        .any(|p| p.status == PlayerStatus::Flip7)
        || state
            .players
            .iter()
            .all(|p| p.status != PlayerStatus::Active)
}

/// Tallies round scores, returns all hand cards to the deck, and prepares the
/// next round.
///
/// Cards flow:
/// 1. Every player's held number/modifier cards are returned to the discard pile
///    so no cards leak out of the deck model across rounds.
/// 2. The discard pile (now containing ALL played cards) is shuffled and placed
///    at the **bottom** of the draw pile — undrawn cards stay on top and are
///    seen first next round, matching the real-game behaviour.
///
/// Only call this when `state.phase == GamePhase::RoundOver`.
pub fn end_round(state: &mut GameState) {
    for i in 0..state.players.len() {
        // Score non-busted players.
        if state.players[i].status != PlayerStatus::Busted {
            let round_score = calculate_score(&state.players[i]);
            state.players[i].total_score += round_score;
        }

        // Return every card in the player's hand to the discard pile
        // so the deck model stays complete across rounds.
        for &n in &state.players[i].numbers {
            state.deck.discard_pile.push(Card::Number(n));
        }
        for m in state.players[i].modifiers.clone() {
            state.deck.discard_pile.push(Card::Modifier(m));
        }
        if state.players[i].has_lifeline {
            state
                .deck
                .discard_pile
                .push(Card::Action(ActionCard::Lifeline));
        }

        state.players[i].numbers.clear();
        state.players[i].modifiers.clear();
        state.players[i].has_lifeline = false;
        state.players[i].status = PlayerStatus::Active;
    }

    state.round += 1;

    // Shuffle the discard pile and place it at the BOTTOM of the draw pile.
    // (Vec::pop draws from the end = "top", so prepending = bottom.)
    let mut recycled: Vec<Card> = state.deck.discard_pile.drain(..).collect();
    shuffle_deck(&mut recycled);
    recycled.append(&mut state.deck.draw_pile); // draw_pile on top (end of vec)
    state.deck.draw_pile = recycled;

    state.pending_effect = None;
    state.effect_queue.clear();
    state.current_player = 0;
    state.turn_holder = 0;
    state.phase = GamePhase::Playing;
}

// ── Turn management ───────────────────────────────────────────────────────────

/// Advances `current_player` to the next `Active` player.
///
/// Returns `true` if an active player was found, `false` if no active players
/// remain (caller should treat the round as over).
pub fn next_turn(state: &mut GameState) -> bool {
    let count = state.players.len();
    let start = state.current_player;
    loop {
        state.current_player = (state.current_player + 1) % count;
        if state.players[state.current_player].status == PlayerStatus::Active {
            // A genuine new turn starts — sync the turn holder.
            state.turn_holder = state.current_player;
            return true;
        }
        // Wrapped all the way around without finding an active player.
        if state.current_player == start {
            return false;
        }
    }
}

// ── Deck helpers ──────────────────────────────────────────────────────────────

/// Shuffles the discard pile and appends it to the bottom of the draw pile
/// when the draw pile is exhausted mid-round.
pub fn reshuffle_if_needed(state: &mut GameState) {
    if state.deck.draw_pile.is_empty() && !state.deck.discard_pile.is_empty() {
        let mut recycled: Vec<Card> = state.deck.discard_pile.drain(..).collect();
        shuffle_deck(&mut recycled);
        // draw_pile is empty here so this is equivalent to replacing it,
        // but the pattern mirrors end_round for consistency.
        recycled.append(&mut state.deck.draw_pile);
        state.deck.draw_pile = recycled;
    }
}
