use wasm_bindgen::prelude::*;

use engine::{
    game::{
        actions::Action,
        engine::apply_action as engine_apply,
        state::GameState,
        turns::{end_round as engine_end_round, new_game as engine_new_game},
    },
    simulation::{bust_probability_for_current_player, recommend as engine_recommend},
};

// ── Serialization helpers ─────────────────────────────────────────────────────

fn to_js<T: serde::Serialize>(val: &T) -> Result<JsValue, JsError> {
    serde_wasm_bindgen::to_value(val).map_err(|e| JsError::new(&e.to_string()))
}

fn from_js<T: serde::de::DeserializeOwned>(val: JsValue) -> Result<T, JsError> {
    serde_wasm_bindgen::from_value(val).map_err(|e| JsError::new(&e.to_string()))
}

// ── Exported functions ────────────────────────────────────────────────────────

/// Creates a new game state for `player_count` players.
///
/// ```js
/// const state = new_game(4);
/// ```
#[wasm_bindgen]
pub fn new_game(player_count: usize) -> Result<JsValue, JsError> {
    to_js(&engine_new_game(player_count))
}

/// Applies one action and returns the updated game state.
///
/// Throws a JS `Error` on invalid actions (wrong turn, pending effect, etc.).
///
/// ```js
/// // Random draw (simulation)
/// state = apply_action(state, state.current_player, { type: "Draw" });
///
/// // Known draw (real-game tracking)
/// state = apply_action(state, state.current_player, {
///   type: "DrawKnown",
///   card: { type: "Number", value: 7 }
/// });
///
/// // Stop
/// state = apply_action(state, state.current_player, { type: "Stop" });
///
/// // Resolve Freeze / Tap3
/// state = apply_action(state, state.current_player, { type: "Freeze", target: 2 });
/// state = apply_action(state, state.current_player, { type: "Tap3",   target: 1 });
/// ```
#[wasm_bindgen]
pub fn apply_action(state: JsValue, player_id: usize, action: JsValue) -> Result<JsValue, JsError> {
    let state: GameState = from_js(state)?;
    let action: Action = from_js(action)?;
    let new_state =
        engine_apply(state, player_id, action).map_err(|e| JsError::new(&format!("{e:?}")))?;
    to_js(&new_state)
}

/// Runs a Monte Carlo simulation and returns a `Recommendation` object,
/// or `null` when the current player is not active or the phase is not Playing.
///
/// `n_simulations` — number of random game completions to average.
/// 500–1000 is a good quality/speed trade-off.
///
/// ```js
/// const rec = get_recommendation(state, 500);
/// if (rec) {
///   console.log(rec.action);            // { type: "Draw" | "Stop" | "Target", index?: number }
///   console.log(rec.expected_score_draw); // number | null
///   console.log(rec.expected_score_stop); // number | null
///   console.log(rec.bust_probability);    // 0.0 – 1.0
/// }
/// ```
#[wasm_bindgen]
pub fn get_recommendation(state: JsValue, n_simulations: u32) -> Result<JsValue, JsError> {
    let state: GameState = from_js(state)?;
    to_js(&engine_recommend(&state, n_simulations))
}

/// Returns the fraction (0.0–1.0) of cards remaining in the draw pile that
/// would bust the current player on the next draw.
///
/// Cheap analytical calculation — no simulation required.
/// Great for a live risk indicator that updates on every card draw.
///
/// ```js
/// const risk = get_bust_probability(state); // e.g. 0.127 → 12.7%
/// ```
#[wasm_bindgen]
pub fn get_bust_probability(state: JsValue) -> Result<f64, JsError> {
    let state: GameState = from_js(state)?;
    Ok(bust_probability_for_current_player(&state))
}

/// Tallies round scores, returns all hand cards to the deck, and advances to
/// the next round.  Only call this when `state.phase.type === "RoundOver"`.
///
/// ```js
/// if (state.phase.type === "RoundOver") {
///   state = end_round(state);
/// }
/// ```
#[wasm_bindgen]
pub fn end_round(state: JsValue) -> Result<JsValue, JsError> {
    let mut state: GameState = from_js(state)?;
    engine_end_round(&mut state);
    to_js(&state)
}
