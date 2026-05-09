use rand::{prelude::IndexedRandom, rng};
use serde::{Deserialize, Serialize};

use crate::game::{
    actions::Action,
    cards::Card,
    deck::shuffle_deck,
    engine::apply_action,
    scoring::calculate_score,
    state::{GamePhase, GameState, PendingEffect, PlayerStatus},
};

// ── Public types ──────────────────────────────────────────────────────────────

/// The recommended action for the current player.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RecommendedAction {
    Draw,
    Stop,
    /// Best player index to target with the pending Freeze or Tap3.
    /// JS shape: `{ "type": "Target", "index": 2 }`
    Target {
        index: usize,
    },
}

/// Result of a Monte Carlo recommendation query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// The suggested action.
    pub action: RecommendedAction,
    /// Average round score for the current player across all "draw" simulations.
    /// `None` when a pending effect is being resolved (Draw/Stop irrelevant).
    pub expected_score_draw: Option<f64>,
    /// Average round score for the current player across all "stop" simulations.
    /// `None` when a pending effect is being resolved.
    pub expected_score_stop: Option<f64>,
    /// Fraction of remaining draw-pile cards that would cause a bust on the
    /// next draw. Useful to display a live risk indicator in the UI.
    pub bust_probability: f64,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Recommends the best action for `state.current_player` using Monte Carlo
/// simulation.
///
/// Call this with the **current `GameState`** — whether that was built by
/// tracking a real game with `Action::DrawKnown` or entirely through
/// simulation. The deck model in the state is used directly, so the more
/// accurately you have tracked the real game, the better the recommendation.
///
/// Returns `None` when:
/// - The game phase is not `Playing`.
/// - The current player is not `Active` (already stopped/busted).
///
/// `n_simulations` of 500–1000 gives a good quality/speed trade-off.
pub fn recommend(state: &GameState, n_simulations: u32) -> Option<Recommendation> {
    if state.phase != GamePhase::Playing {
        return None;
    }
    let player_id = state.current_player;
    if state.players[player_id].status != PlayerStatus::Active {
        return None;
    }

    let bust_prob = bust_probability(state, player_id);

    // If a pending Freeze/Tap3 needs a target, skip Draw/Stop analysis.
    if state.pending_effect.is_some() {
        let target = best_effect_target(state, n_simulations);
        return Some(Recommendation {
            action: RecommendedAction::Target { index: target },
            expected_score_draw: None,
            expected_score_stop: None,
            bust_probability: bust_prob,
        });
    }

    let score_draw = avg_score_after(state, player_id, Action::Draw, n_simulations);
    let score_stop = avg_score_after(state, player_id, Action::Stop, n_simulations);

    Some(Recommendation {
        action: if score_draw >= score_stop {
            RecommendedAction::Draw
        } else {
            RecommendedAction::Stop
        },
        expected_score_draw: Some(score_draw),
        expected_score_stop: Some(score_stop),
        bust_probability: bust_prob,
    })
}

/// Returns the fraction of cards currently in the draw pile that would
/// cause the current player to bust on the next draw.
///
/// You can call this directly to display a live risk indicator without running
/// a full recommendation.
pub fn bust_probability_for_current_player(state: &GameState) -> f64 {
    bust_probability(state, state.current_player)
}

// ── Monte Carlo internals ─────────────────────────────────────────────────────

/// Applies `action` from `state` and averages the round score of `player_id`
/// over `n` independent random game completions.
fn avg_score_after(state: &GameState, player_id: usize, action: Action, n: u32) -> f64 {
    let mut total = 0i64;
    let mut count = 0u32;

    for _ in 0..n {
        if let Ok(new_state) = apply_action(state.clone(), player_id, action.clone()) {
            total += simulate_to_round_end(new_state, player_id) as i64;
            count += 1;
        }
    }

    if count == 0 {
        0.0
    } else {
        total as f64 / count as f64
    }
}

/// For a pending Freeze/Tap3, returns the player index that maximises the
/// current player's expected round score.
fn best_effect_target(state: &GameState, n_simulations: u32) -> usize {
    let player_id = state.current_player;
    let effect = state
        .pending_effect
        .as_ref()
        .expect("best_effect_target called without a pending effect");

    let mut best_idx = player_id; // safe fallback (self-target ignored below)
    let mut best_score = f64::NEG_INFINITY;

    for target in 0..state.players.len() {
        if target == player_id {
            continue;
        }
        if state.players[target].status != PlayerStatus::Active {
            continue;
        }

        let action = match effect {
            PendingEffect::Freeze => Action::Freeze { target },
            PendingEffect::Tap3 => Action::Tap3 { target },
        };

        let avg = avg_score_after(state, player_id, action, n_simulations);
        if avg > best_score {
            best_score = avg;
            best_idx = target;
        }
    }

    best_idx
}

/// Plays out the rest of the current round from `state` using a simple
/// heuristic for every player and returns the round score of `target_player`.
///
/// The draw pile is **re-shuffled** at the start of every call so that
/// independent simulation runs explore different card orderings.
fn simulate_to_round_end(mut state: GameState, target_player: usize) -> i32 {
    // Re-randomise card order for this simulation run.
    shuffle_deck(&mut state.deck.draw_pile);

    let mut r = rng();
    const MAX_STEPS: u32 = 500; // guard against any unexpected infinite loop

    for _ in 0..MAX_STEPS {
        if state.phase == GamePhase::RoundOver {
            break;
        }

        let current = state.current_player;

        // Resolve pending Freeze/Tap3 with a random active opponent.
        if state.pending_effect.is_some() {
            match random_active_opponent(&state, current, &mut r) {
                Some(t) => {
                    let action = match state.pending_effect.as_ref().unwrap() {
                        PendingEffect::Freeze => Action::Freeze { target: t },
                        PendingEffect::Tap3 => Action::Tap3 { target: t },
                    };
                    if let Ok(s) = apply_action(state.clone(), current, action) {
                        state = s;
                    } else {
                        state.pending_effect = None;
                    }
                }
                None => {
                    // No valid target — discard the effect.
                    state.pending_effect = None;
                }
            }
            continue;
        }

        let action = if heuristic_should_draw(&state, current) {
            Action::Draw
        } else {
            Action::Stop
        };

        match apply_action(state.clone(), current, action) {
            Ok(s) => state = s,
            Err(_) => break,
        }
    }

    round_score_of(&state, target_player)
}

// ── Heuristic & probability helpers ──────────────────────────────────────────

/// Simple draw heuristic used for all simulated players:
///
/// | Unique numbers held | Strategy |
/// |---|---|
/// | 0 – 3 | Always draw — low bust risk |
/// | 4 – 5 | Draw only if bust probability < 30 % |
/// | 6 | Draw unless bust probability ≥ 60 % — the Flip7 bonus makes it worth it |
/// | 7+ | Never (already at Flip7, shouldn't reach this branch) |
fn heuristic_should_draw(state: &GameState, player_id: usize) -> bool {
    let n = state.players[player_id].numbers.len();
    match n {
        0..=3 => true,
        4..=5 => bust_probability(state, player_id) < 0.30,
        6 => bust_probability(state, player_id) < 0.60,
        _ => false,
    }
}

/// Fraction of cards currently in the draw pile that would bust `player_id`.
fn bust_probability(state: &GameState, player_id: usize) -> f64 {
    let pile = &state.deck.draw_pile;
    if pile.is_empty() {
        return 0.0;
    }
    let held = &state.players[player_id].numbers;
    let busting = pile
        .iter()
        .filter(|c| matches!(c, Card::Number(n) if held.contains(n)))
        .count();
    busting as f64 / pile.len() as f64
}

/// Returns the round score for `player_id` (0 if they busted).
fn round_score_of(state: &GameState, player_id: usize) -> i32 {
    if state.players[player_id].status == PlayerStatus::Busted {
        0
    } else {
        calculate_score(&state.players[player_id])
    }
}

/// Picks a uniformly random player that is `Active` and is not `exclude`.
fn random_active_opponent(
    state: &GameState,
    exclude: usize,
    r: &mut impl rand::Rng,
) -> Option<usize> {
    let candidates: Vec<usize> = state
        .players
        .iter()
        .enumerate()
        .filter(|&(i, p)| i != exclude && p.status == PlayerStatus::Active)
        .map(|(i, _)| i)
        .collect();
    candidates.choose(r).copied()
}
