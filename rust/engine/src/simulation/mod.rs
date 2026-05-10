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

    // ── Debug fields (temporary) ────────────────────────────────────────────
    /// How many of the N Monte Carlo simulations for Draw completed successfully.
    /// 0 means every `apply_action(Draw)` call returned an error — points to a
    /// state inconsistency (wrong phase, pending effect not detected, etc.).
    pub debug_sim_count: u32,
    /// Whether the bust-probability heuristic gate passed (true = drawing is
    /// not considered too risky for this card count).
    pub debug_heuristic_pass: bool,
    /// The risk margin added to score_stop before comparing against score_draw.
    pub debug_risk_margin: f64,
    /// Number of unique number cards the current player is holding.
    pub debug_numbers_held: usize,
    /// The raw (pre-margin) score_draw value from the simulation.
    pub debug_raw_score_draw: f64,
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
            debug_sim_count: 0,
            debug_heuristic_pass: false,
            debug_risk_margin: 0.0,
            debug_numbers_held: state.players[player_id].numbers.len(),
            debug_raw_score_draw: 0.0,
        });
    }

    let (score_draw, sim_count) = avg_score_after(state, player_id, Action::Draw, n_simulations);
    // Stopping is deterministic — the player's score is already known exactly.
    // Running 500 simulations for it wastes half the budget and adds no information.
    let score_stop = calculate_score(&state.players[player_id]) as f64;

    // Risk-adjusted comparison: require the expected draw gain to outweigh a
    // fraction of what you stand to lose by busting from your current position.
    //
    //   margin = bust_probability × current_score × RISK_FACTOR
    //
    // A pure EV approach (RISK_FACTOR = 0) recommends Draw whenever E[draw] > E[stop]
    // by even 0.001 pts, ignoring the cost of losing a good score on a bust.
    // With RISK_FACTOR = 0.5 the advisor requires a meaningful expected gain before
    // recommending the risky option, better matching how a cautious player thinks.
    const RISK_FACTOR: f64 = 0.5;
    let risk_margin = bust_prob * score_stop * RISK_FACTOR;
    let heuristic_pass = heuristic_should_draw(state, player_id);

    // Apply the same bust-probability gates used for every simulated player.
    // If the threshold says the raw bust risk is already too high for your card
    // count, recommend Stop unconditionally — no EV justification overrides it.
    // This keeps the real-player recommendation consistent with the simulation
    // heuristic and prevents the EV math from talking you into clearly bad draws.
    let action = if !heuristic_pass {
        RecommendedAction::Stop
    } else if score_draw > score_stop + risk_margin {
        RecommendedAction::Draw
    } else {
        RecommendedAction::Stop
    };

    Some(Recommendation {
        action,
        expected_score_draw: Some(score_draw),
        expected_score_stop: Some(score_stop),
        bust_probability: bust_prob,
        debug_sim_count: sim_count,
        debug_heuristic_pass: heuristic_pass,
        debug_risk_margin: risk_margin,
        debug_numbers_held: state.players[player_id].numbers.len(),
        debug_raw_score_draw: score_draw,
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
/// Returns `(average_score, successful_sim_count)`. A count of 0 means every
/// `apply_action` call failed — the caller should treat the score as invalid.
///
/// **Critical**: the draw pile is shuffled before every `apply_action` so that
/// each simulation draws a *different* first card. Without this every run pops
/// the same top card and the Monte Carlo degenerates to a single outcome
/// repeated N times (std = 0, bust rate ignores actual probability).
fn avg_score_after(state: &GameState, player_id: usize, action: Action, n: u32) -> (f64, u32) {
    let mut total = 0i64;
    let mut count = 0u32;

    for _ in 0..n {
        let mut sim_state = state.clone();
        shuffle_deck(&mut sim_state.deck.draw_pile);
        if let Ok(new_state) = apply_action(sim_state, player_id, action.clone()) {
            total += simulate_to_round_end(new_state, player_id) as i64;
            count += 1;
        }
    }

    let avg = if count == 0 {
        0.0
    } else {
        total as f64 / count as f64
    };
    (avg, count)
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

        let (avg, _) = avg_score_after(state, player_id, action, n_simulations);
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

/// Draw heuristic used for all simulated players.
///
/// This heuristic controls how aggressively future turns are played inside
/// `simulate_to_round_end`. Because `avg_score_after(Draw)` evaluates the
/// current draw by simulating future turns with this policy, an over-aggressive
/// heuristic inflates E[draw] and biases the top-level recommendation toward
/// drawing. The thresholds below are calibrated to represent a realistic,
/// slightly conservative player:
///
/// | Unique numbers held | Draw if bust probability is below… | Rationale |
/// |---|---|---|
/// | 0 – 3 | always | Very low bust risk; every new number adds value |
/// | 4 – 5 | 25 % | Moderate risk; protect a growing score |
/// | 6     | 35 % | Flip7 bonus is tempting but 60 % was far too aggressive |
/// | 7+    | never | Already at Flip7 — turn is over |
fn heuristic_should_draw(state: &GameState, player_id: usize) -> bool {
    let bust_prob = bust_probability(state, player_id);
    match state.players[player_id].numbers.len() {
        0..=3 => true,
        4..=5 => bust_prob < 0.25,
        6 => bust_prob < 0.35,
        _ => false,
    }
}

/// Fraction of cards currently in the draw pile that would bust `player_id`.
///
/// Returns **0.0** when the player holds a Lifeline: a duplicate draw would
/// consume the lifeline instead of causing a bust, so the effective probability
/// of actually losing your score on the next card is zero.
fn bust_probability(state: &GameState, player_id: usize) -> f64 {
    // Lifeline absorbs one duplicate — you cannot bust on this draw.
    if state.players[player_id].has_lifeline {
        return 0.0;
    }
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

// ── Debug / analysis helpers (public) ──────────────────────────────────────

/// Returns the individual round scores of `player_id` over `n` independent
/// simulations, applying `action` first and then playing the rest of the round
/// with the internal heuristic.
///
/// Unlike [`recommend`], which returns only the mean, this gives callers the
/// full distribution so they can build histograms, compute percentiles, and
/// reason about variance and tail risk.
///
/// Returns an empty `Vec` when `action` is invalid for the current state
/// (e.g. wrong phase, pending effect unresolved).
pub fn score_distribution(state: &GameState, player_id: usize, action: Action, n: u32) -> Vec<i32> {
    let mut scores = Vec::with_capacity(n as usize);
    for _ in 0..n {
        let mut sim_state = state.clone();
        // Shuffle before every simulation — same fix as in avg_score_after.
        shuffle_deck(&mut sim_state.deck.draw_pile);
        match apply_action(sim_state, player_id, action.clone()) {
            Ok(new_state) => scores.push(simulate_to_round_end(new_state, player_id)),
            Err(_) => {}
        }
    }
    scores
}

/// Returns the bust probability for any `player_id`, not just `current_player`.
///
/// Useful when iterating over all active players to compare their risk
/// profiles without having to temporarily reassign `current_player`.
pub fn bust_probability_for_player(state: &GameState, player_id: usize) -> f64 {
    bust_probability(state, player_id)
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
