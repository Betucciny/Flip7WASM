/**
 * engine.ts — Typed wrapper around the Flip7 WASM bindings.
 *
 * Import this file instead of the raw @wasm/wasm module so you get
 * full TypeScript types for every function and data structure.
 *
 * Usage:
 *   import { initEngine, newGame, applyAction, getRecommendation } from "@app/engine";
 *
 *   await initEngine();
 *   let state = newGame(4);
 *   state = applyAction(state, state.current_player, { type: "Draw" });
 *   const rec = getRecommendation(state, 500);
 */

import init, {
  new_game,
  apply_action,
  get_recommendation,
  get_bust_probability,
  end_round,
} from "@wasm/wasm";

// ── Card types ────────────────────────────────────────────────────────────────

export type ActionCardKind =
  | { type: "Lifeline" }
  | { type: "Freeze" }
  | { type: "Tap3" };

export type ModifierCard =
  | { type: "Add"; value: number }
  | { type: "Multiply2" };

export type Card =
  | { type: "Number"; value: number }
  | { type: "Action"; value: ActionCardKind }
  | { type: "Modifier"; value: ModifierCard };

// ── Player / game state ───────────────────────────────────────────────────────

export type PlayerStatus =
  | { type: "Active" }
  | { type: "Stopped" }
  | { type: "Busted" }
  | { type: "Flip7" };

export type PendingEffect = { type: "Freeze" } | { type: "Tap3" };

export type GamePhase = { type: "Playing" } | { type: "RoundOver" };

export interface Player {
  numbers: number[];
  modifiers: ModifierCard[];
  has_lifeline: boolean;
  status: PlayerStatus;
  total_score: number;
}

export interface Deck {
  draw_pile: Card[];
  discard_pile: Card[];
}

export interface GameState {
  players: Player[];
  deck: Deck;
  current_player: number;
  /** The player whose actual turn it is. Equal to current_player during normal
   *  play; differs when a Tap3 victim is temporarily resolving chained effects
   *  they drew during their forced draws. */
  turn_holder: number;
  round: number;
  pending_effect: PendingEffect | null;
  /** Effects queued during a Tap3 forced-draw sequence, to be resolved by the
   *  victim after all 3 draws complete. Each entry is [resolver_index, effect]. */
  effect_queue: [number, PendingEffect][];
  phase: GamePhase;
}

// ── Actions ───────────────────────────────────────────────────────────────────

export type Action =
  /** Random draw — engine picks the top card of the shuffled deck. */
  | { type: "Draw" }
  /** Tracked draw — tell the engine exactly which card was flipped. */
  | { type: "DrawKnown"; card: Card }
  /** Lock in the current player's score and end their turn. */
  | { type: "Stop" }
  /** Resolve a pending Freeze card by choosing a target. */
  | { type: "Freeze"; target: number }
  /** Resolve a pending Tap3 card — engine draws 3 random cards for the target. */
  | { type: "Tap3"; target: number }
  /** Resolve a pending Tap3 card with known cards (real-game tracking).
   *  Supply the exact cards the target actually drew so the deck model stays accurate. */
  | { type: "Tap3Known"; target: number; cards: Card[] };

// ── Recommendation ────────────────────────────────────────────────────────────

export type RecommendedAction =
  | { type: "Draw" }
  | { type: "Stop" }
  /** Best target player for a pending Freeze or Tap3 effect. */
  | { type: "Target"; index: number };

export interface Recommendation {
  action: RecommendedAction;
  /** Average round score across simulations if the player draws now. null when resolving an effect. */
  expected_score_draw: number | null;
  /** Average round score across simulations if the player stops now. null when resolving an effect. */
  expected_score_stop: number | null;
  /** Fraction of remaining draw-pile cards that would bust the current player (0.0 – 1.0). */
  bust_probability: number;
}

// ── Init ──────────────────────────────────────────────────────────────────────

/** Must be called (and awaited) once before using any other function. */
export async function initEngine(): Promise<void> {
  await init();
}

// ── Typed wrappers ────────────────────────────────────────────────────────────

/** Start a new game with the given number of players (2–6). */
export function newGame(playerCount: number): GameState {
  return new_game(playerCount) as GameState;
}

/**
 * Apply one action on behalf of `playerId` and return the new state.
 * Throws if the action is illegal (wrong turn, pending effect unresolved, etc.).
 */
export function applyAction(
  state: GameState,
  playerId: number,
  action: Action,
): GameState {
  return apply_action(state, playerId, action) as GameState;
}

/**
 * Run a Monte Carlo recommendation for the current player.
 * Returns `null` if the game phase is not Playing or the player is inactive.
 *
 * @param nSimulations  How many random game completions to average. 500 is fast; 1000 is more accurate.
 */
export function getRecommendation(
  state: GameState,
  nSimulations = 500,
): Recommendation | null {
  return get_recommendation(state, nSimulations) as Recommendation | null;
}

/**
 * Fraction of remaining draw-pile cards that would bust the current player.
 * Instant analytical calculation — no simulation cost.
 * Use this for a live bust-risk indicator that refreshes every draw.
 */
export function getBustProbability(state: GameState): number {
  return get_bust_probability(state) as number;
}

/**
 * Tally scores, return hand cards to the deck, and advance to the next round.
 * Only call this when `state.phase.type === "RoundOver"`.
 */
export function endRound(state: GameState): GameState {
  return end_round(state) as GameState;
}
