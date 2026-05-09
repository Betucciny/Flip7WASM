import type { Player } from "@app/engine";

/**
 * Mirrors the Rust `calculate_score` function exactly.
 *
 * Evaluation order:
 * 1. Sum all number cards.
 * 2. Apply all Add modifiers.
 * 3. Apply Multiply2 (if present) — order of receipt doesn't matter.
 * 4. Add +15 Flip7 bonus when the player holds 7+ unique numbers (applied
 *    after the multiplier so it is not doubled).
 *
 * Busted players always contribute 0 to round totals; callers are responsible
 * for that check.
 */
export function calculateScore(player: Player): number {
  let score = player.numbers.reduce((a, b) => a + b, 0);

  for (const mod of player.modifiers) {
    if (mod.type === "Add") score += mod.value;
  }
  for (const mod of player.modifiers) {
    if (mod.type === "Multiply2") score *= 2;
  }
  if (player.numbers.length >= 7) score += 15;

  return score;
}

/** Round score for display: 0 if busted, calculateScore otherwise. */
export function roundScore(player: Player): number {
  return player.status.type === "Busted" ? 0 : calculateScore(player);
}

/** Returns the index of the player with the highest total_score >= 200, or -1. */
export function findWinner(players: Player[]): number {
  let best = -1;
  let bestScore = -1;
  for (let i = 0; i < players.length; i++) {
    const p = players[i];
    if (p.total_score >= 200 && p.total_score > bestScore) {
      best = i;
      bestScore = p.total_score;
    }
  }
  return best;
}
