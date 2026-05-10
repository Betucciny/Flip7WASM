use crate::game::actions::Action;
use crate::game::cards::{ActionCard, Card};
use crate::game::state::{GamePhase, GameState, PendingEffect, PlayerStatus};
use crate::game::turns::{is_round_over, next_turn, reshuffle_if_needed};

// ── Public API ────────────────────────────────────────────────────────────────

/// Applies `action` on behalf of `player_id` and returns the updated state.
///
/// This function is **pure**: it takes ownership of the state and returns a new
/// one, making it safe to clone and branch freely for simulation.
///
/// # Modes of use
///
/// **Simulation** — use `Action::Draw`. The engine pops a random card from its
/// internal shuffled draw pile.
///
/// **Real-game tracking** — use `Action::DrawKnown(card)`. Supply the exact
/// card that was flipped in the physical game. The engine removes that card
/// from the deck model so the remaining draw pile always reflects the true set
/// of unseen cards, giving simulations run from this state accurate
/// probabilities.
///
/// # Errors
/// - [`EngineError::InvalidAction`] — wrong phase, wrong player's turn, or
///   attempting to `Draw`/`Stop` while a pending effect awaits resolution.
/// - [`EngineError::InvalidTarget`] — target index is out of bounds.
/// - [`EngineError::PendingEffectUnresolved`] — a Freeze/Tap3 was drawn and
///   the player must choose a target before doing anything else.
pub fn apply_action(
    state: GameState,
    player_id: usize,
    action: Action,
) -> Result<GameState, EngineError> {
    let mut state = state;

    if state.phase != GamePhase::Playing {
        return Err(EngineError::InvalidAction);
    }

    if player_id != state.current_player {
        return Err(EngineError::InvalidAction);
    }

    // Gate Draw/DrawKnown/Stop behind pending-effect resolution.
    match &action {
        Action::Draw | Action::DrawKnown { .. } | Action::Stop => {
            if state.pending_effect.is_some() {
                return Err(EngineError::PendingEffectUnresolved);
            }
        }
        Action::Freeze { .. } => {
            if state.pending_effect != Some(PendingEffect::Freeze) {
                return Err(EngineError::InvalidAction);
            }
        }
        Action::Tap3 { .. } => {
            if state.pending_effect != Some(PendingEffect::Tap3) {
                return Err(EngineError::InvalidAction);
            }
        }
        Action::Tap3Known { .. } => {
            if state.pending_effect != Some(PendingEffect::Tap3) {
                return Err(EngineError::InvalidAction);
            }
        }
    }

    match action {
        // ── Random draw (simulation) ─────────────────────────────────────────
        Action::Draw => {
            draw_card(&mut state, player_id);
            handle_post_draw(&mut state, player_id);
        }

        // ── Known draw (real-game tracking) ──────────────────────────────────
        // Remove the card from the deck model first so the remaining draw pile
        // reflects reality, then apply the card's effect normally.
        Action::DrawKnown { card } => {
            remove_known_card(&mut state, &card);
            apply_card_effect(&mut state, player_id, card);
            handle_post_draw(&mut state, player_id);
        }

        Action::Stop => {
            stop_player(&mut state, player_id);
            advance_or_end(&mut state);
        }

        Action::Freeze { target } => {
            if target >= state.players.len() {
                return Err(EngineError::InvalidTarget);
            }
            freeze_player(&mut state, target);
            state.pending_effect = None;
            advance_after_effect(&mut state);
        }

        Action::Tap3 { target } => {
            if target >= state.players.len() {
                return Err(EngineError::InvalidTarget);
            }
            // Clear the triggering Tap3 BEFORE the forced draws so that the
            // `.take()` inside the loop only captures effects drawn *during*
            // those 3 forced cards, not the original pending effect.
            state.pending_effect = None;
            tap3(&mut state, target);
            after_tap3(&mut state, target);
        }

        Action::Tap3Known { target, cards } => {
            if target >= state.players.len() {
                return Err(EngineError::InvalidTarget);
            }
            // Same fix: clear before the forced draws.
            state.pending_effect = None;
            tap3_known(&mut state, target, cards);
            after_tap3(&mut state, target);
        }
    }

    Ok(state)
}

// ── Drawing helpers ───────────────────────────────────────────────────────────

/// Pops the top card of the draw pile (reshuffling first if needed) and
/// applies its effect.
fn draw_card(state: &mut GameState, player_id: usize) {
    reshuffle_if_needed(state);
    let card = match state.deck.draw_pile.pop() {
        Some(c) => c,
        None => return, // empty even after reshuffle — extremely rare, safe to skip
    };
    apply_card_effect(state, player_id, card);
}

/// Applies the effect of `card` to `player_id`.
/// The card is assumed to have already been removed from the draw pile by the
/// caller (either via `draw_card` or `remove_known_card`).
fn apply_card_effect(state: &mut GameState, player_id: usize, card: Card) {
    match card {
        Card::Number(n) => handle_number_card(state, player_id, n),
        Card::Modifier(m) => state.players[player_id].modifiers.push(m),
        Card::Action(a) => {
            resolve_action_card(state, player_id, a.clone());
            state.deck.discard_pile.push(Card::Action(a));
        }
    }
}

/// Searches the draw pile for the first card matching `card` (from the top)
/// and removes it. Falls back to the discard pile in case of out-of-sync state.
fn remove_known_card(state: &mut GameState, card: &Card) {
    if let Some(pos) = state.deck.draw_pile.iter().rposition(|c| c == card) {
        state.deck.draw_pile.remove(pos);
        return;
    }
    // Fallback: card might already be in the discard pile (e.g. after a
    // mid-round reshuffle that wasn't tracked by the caller).
    if let Some(pos) = state.deck.discard_pile.iter().rposition(|c| c == card) {
        state.deck.discard_pile.remove(pos);
    }
}

// ── Card-effect helpers ───────────────────────────────────────────────────────

/// Processes a number card draw:
/// - Duplicate + lifeline → the duplicate and the lifeline are discarded, turn
///   continues (the player is still `Active`).
/// - Duplicate + no lifeline → player busts; the card is still discarded.
/// - New number → added to hand; triggers `Flip7` if this is the 7th unique
///   number.
fn handle_number_card(state: &mut GameState, player_id: usize, number: u8) {
    if state.players[player_id].numbers.contains(&number) {
        // Duplicate: always discard the card.
        state.deck.discard_pile.push(Card::Number(number));
        if state.players[player_id].has_lifeline {
            // Lifeline saves the player.
            state.players[player_id].has_lifeline = false;
            state
                .deck
                .discard_pile
                .push(Card::Action(ActionCard::Lifeline));
            // Player remains Active; their turn continues.
        } else {
            state.players[player_id].status = PlayerStatus::Busted;
        }
        return;
    }

    state.players[player_id].numbers.push(number);
    if state.players[player_id].numbers.len() >= 7 {
        state.players[player_id].status = PlayerStatus::Flip7;
    }
}

fn resolve_action_card(state: &mut GameState, player_id: usize, action: ActionCard) {
    match action {
        // Lifeline goes to whoever drew it — not necessarily current_player
        // (e.g. a Tap3 victim can draw a lifeline).
        ActionCard::Lifeline => {
            state.players[player_id].has_lifeline = true;
        }
        ActionCard::Freeze => {
            state.pending_effect = Some(PendingEffect::Freeze);
        }
        ActionCard::Tap3 => {
            state.pending_effect = Some(PendingEffect::Tap3);
        }
    }
}

fn stop_player(state: &mut GameState, player_id: usize) {
    state.players[player_id].status = PlayerStatus::Stopped;
}

fn freeze_player(state: &mut GameState, target: usize) {
    if state.players[target].status == PlayerStatus::Active {
        state.players[target].status = PlayerStatus::Stopped;
    }
}

/// Forces `target` to draw up to 3 cards, stopping early on bust or Flip7.
/// Any Freeze/Tap3 the target draws are **queued** in `state.effect_queue`
/// rather than immediately resolved — they are resolved after all 3 forced
/// draws complete, per the official rules.
fn tap3(state: &mut GameState, target: usize) {
    for _ in 0..3 {
        if state.players[target].status != PlayerStatus::Active {
            break;
        }
        draw_card(state, target);
        // Move any pending effect (Freeze/Tap3 drawn this card) to the queue.
        if let Some(effect) = state.pending_effect.take() {
            state.effect_queue.push((target, effect));
        }
    }
}

/// Real-game-tracking variant of `tap3`: applies known cards instead of drawing
/// randomly, so the deck model stays accurate for future simulations.
/// Chained effects are queued exactly as in `tap3`.
fn tap3_known(state: &mut GameState, target: usize, cards: Vec<Card>) {
    for card in cards {
        if state.players[target].status != PlayerStatus::Active {
            break;
        }
        remove_known_card(state, &card);
        apply_card_effect(state, target, card);
        if let Some(effect) = state.pending_effect.take() {
            state.effect_queue.push((target, effect));
        }
    }
}

/// Advances to the next active player, or marks the round as over.
fn advance_or_end(state: &mut GameState) {
    if is_round_over(state) {
        state.phase = GamePhase::RoundOver;
    } else {
        next_turn(state);
    }
}

/// Called after a Tap3 sequence completes.
///
/// If the target busted or reached Flip7 during their forced draws, their
/// queued effects are dropped (they cannot resolve effects when inactive).
/// Then the remaining queue is processed via `advance_after_effect`.
fn after_tap3(state: &mut GameState, target: usize) {
    if state.players[target].status != PlayerStatus::Active {
        // Victim can’t resolve effects they drew while busted / Flip7.
        state
            .effect_queue
            .retain(|(resolver, _)| *resolver != target);
    }
    advance_after_effect(state);
}

/// After any Freeze/Tap3 resolution, pops the next queued chain effect and
/// sets it up for resolution, or — when the queue is empty — restores
/// `current_player` to the turn holder and advances the turn.
///
/// Effects with no valid active target are automatically skipped so the UI
/// never shows an empty target picker.
fn advance_after_effect(state: &mut GameState) {
    // Drain the queue, skipping entries that have no available targets.
    while let Some((resolver, effect)) = state.effect_queue.first().cloned() {
        let has_target = state
            .players
            .iter()
            .enumerate()
            .any(|(i, p)| i != resolver && p.status == PlayerStatus::Active);

        if has_target {
            // Pop and set up this effect for resolution.
            state.effect_queue.remove(0);
            state.current_player = resolver;
            state.pending_effect = Some(effect);
            return;
        } else {
            // No targets available — silently discard this effect.
            state.effect_queue.remove(0);
        }
    }

    // Queue exhausted — restore the original turn holder and advance.
    state.current_player = state.turn_holder;
    advance_or_end(state);
}

/// Called after any draw action.
///
/// One card is drawn per turn — once a card has been resolved and no
/// pending effect (Freeze/Tap3 target choice) is outstanding, the turn
/// always passes to the next player, whether the drawing player is still
/// active or not.
fn handle_post_draw(state: &mut GameState, _player_id: usize) {
    // If a Freeze or Tap3 was just drawn, the player must still pick a
    // target — don't advance yet.
    if state.pending_effect.is_some() {
        return;
    }
    advance_or_end(state);
}

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
pub enum EngineError {
    /// The action is not legal in the current game phase or turn.
    InvalidAction,
    /// The supplied target index is out of bounds.
    InvalidTarget,
    /// A Freeze or Tap3 card was drawn and must be resolved before proceeding.
    PendingEffectUnresolved,
}
