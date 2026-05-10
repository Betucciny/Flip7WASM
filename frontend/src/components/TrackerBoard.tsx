/**
 * TrackerBoard — game board for Tracker mode.
 *
 * All card draws use the Known variants so the deck model stays accurate
 * and the AI advisor gives correct probabilities.
 *
 * Tap3 flow (multi-step):
 *   1. Choose target player.
 *   2. Enter up to 3 cards the target actually drew (CardPicker appears).
 *   3. Confirm → sends Tap3Known to the engine.
 */
import { createSignal, For, Show } from "solid-js";
import type {
  Action,
  Card,
  GameState,
  Player,
  Recommendation,
} from "@app/engine";
import { calculateScore } from "../utils/scoring";
import { BigNumberCard, ModifierTag, SmallNumberCard } from "./shared/Cards";
import { PlayerRow } from "./shared/PlayerRow";
import { AdvisorPanel } from "./shared/AdvisorPanel";
import { RoundOverModal } from "./shared/RoundOverModal";
import CardPicker from "./CardPicker";

interface Props {
  state: GameState;
  recommendation: Recommendation | null;
  onAction: (action: Action) => void;
  onNextRound: () => void;
  onEndGame: () => void;
  canUndo: boolean;
  onUndo: () => void;
}

// ── Local helpers ─────────────────────────────────────────────────────────────

/**
 * Very lightweight local simulation: walks through a list of cards and
 * determines whether the player would bust or reach Flip7, accounting for
 * Lifeline protection. Used only to auto-close the Tap3 picker early.
 */
function simulateCards(
  player: Player,
  drawn: Card[],
): { done: boolean; reason: "bust" | "flip7" | null } {
  let numbers = [...player.numbers];
  let hasLifeline = player.has_lifeline;

  for (const card of drawn) {
    if (card.type !== "Number") continue;
    if (numbers.includes(card.value)) {
      if (hasLifeline) {
        hasLifeline = false; // lifeline consumed, survive
      } else {
        return { done: true, reason: "bust" };
      }
    } else {
      numbers = [...numbers, card.value];
      if (numbers.length >= 7) return { done: true, reason: "flip7" };
    }
  }
  return { done: false, reason: null };
}

// ── Component ─────────────────────────────────────────────────────────────────

export default function TrackerBoard(props: Props) {
  const s = () => props.state;
  const cur = () => s().players[s().current_player];
  const score = () => calculateScore(cur());
  const pending = () => s().pending_effect;
  const isPlaying = () => s().phase.type === "Playing";

  // ── Tap3Known multi-step local state ──
  const [tap3Target, setTap3Target] = createSignal<number | null>(null);
  const [tap3Cards, setTap3Cards] = createSignal<Card[]>([]);

  const isPickingTap3Target = () =>
    pending()?.type === "Tap3" && tap3Target() === null;
  const isCollectingTap3Cards = () =>
    pending()?.type === "Tap3" && tap3Target() !== null;

  const tap3Player = () => {
    const t = tap3Target();
    return t !== null ? s().players[t] : null;
  };

  const tap3Done = () => {
    const p = tap3Player();
    if (!p) return false;
    if (tap3Cards().length >= 3) return true;
    const { done } = simulateCards(p, tap3Cards());
    return done;
  };

  function addTap3Card(card: Card) {
    setTap3Cards((prev) => [...prev, card]);
  }

  function removeTap3Card(idx: number) {
    setTap3Cards((prev) => prev.filter((_, i) => i !== idx));
  }

  function confirmTap3() {
    const t = tap3Target();
    const cards = tap3Cards();
    if (t === null || cards.length === 0) return;
    props.onAction({ type: "Tap3Known", target: t, cards });
    setTap3Target(null);
    setTap3Cards([]);
  }

  /** Other active players — valid Freeze / Tap3 targets. */
  const targets = () =>
    s()
      .players.map((p, i) => ({ p, i }))
      .filter(({ p }) => p.status.type === "Active");

  // ── Tap3Known status label ──
  const tap3Status = () => {
    const p = tap3Player();
    if (!p) return null;
    const { done, reason } = simulateCards(p, tap3Cards());
    if (!done) return null;
    return reason === "bust" ? "💥 Player busted!" : "🏅 FLIP 7!";
  };

  return (
    <div class="min-h-screen bg-gray-950 flex flex-col">
      {/* ── Header ── */}
      <header class="bg-gray-900 border-b border-gray-800 top-0 z-10">
        <div class="max-w-5xl mx-auto px-4 py-3 flex items-center justify-between">
          <div class="flex items-center gap-3">
            <span class="font-black text-lg">
              <span class="text-white">FLIP</span>
              <span class="text-amber-400"> 7</span>
            </span>
            <span class="text-gray-600">·</span>
            <span class="text-gray-400 text-sm">Round {s().round}</span>
            <span class="bg-blue-900/50 text-blue-400 text-xs px-2 py-0.5 rounded-full border border-blue-700/50">
              📋 Tracker
            </span>
          </div>
          <div class="flex items-center gap-3">
            <div class="flex gap-3 text-xs text-gray-500">
              <span>🃏 {s().deck.draw_pile.length} left</span>
              <span>🗑 {s().deck.discard_pile.length} used</span>
            </div>
            <button
              onClick={props.onUndo}
              disabled={!props.canUndo}
              title="Undo last action"
              class="text-xs px-3 py-1.5 rounded-lg bg-gray-800 hover:bg-gray-700 disabled:opacity-30 disabled:cursor-not-allowed text-gray-300 border border-gray-700 transition-colors font-medium"
            >
              ↩ Undo
            </button>
          </div>
        </div>
      </header>

      {/* ── Main grid ── */}
      <div class="flex-1 max-w-5xl mx-auto w-full p-4 grid lg:grid-cols-5 gap-4 items-start">
        {/* Right: action panel */}
        <Show when={isPlaying()}>
          <div class="lg:col-span-2 space-y-4 top-14">
            {/* Chain-resolution context banner */}
            <Show when={s().current_player !== s().turn_holder}>
              <div class="bg-violet-900/30 border border-violet-500/40 rounded-xl px-3 py-2 text-xs text-violet-300">
                ⛓️ <strong>Chain:</strong> P{s().turn_holder}'s Tap3 → P
                {s().current_player} resolves a queued effect
              </div>
            </Show>
            {/* Current player hand */}
            <div class="bg-gray-900 border border-gray-800 rounded-2xl p-4">
              {/* Advisor — hidden while collecting Tap3 cards to save space */}
              <Show when={props.recommendation}>
                {(rec) => (
                  <Show when={!isCollectingTap3Cards()}>
                    <AdvisorPanel rec={rec()} />
                  </Show>
                )}
              </Show>

              {/* ── Freeze target picker ── */}
              <Show when={pending()?.type === "Freeze"}>
                <div class="bg-gray-900 border border-cyan-500/30 rounded-2xl p-4">
                  <p class="text-sm text-center text-gray-400 mb-3">
                    🧊 Choose a player to{" "}
                    <strong class="text-white">Freeze</strong>
                  </p>
                  <div class="grid grid-cols-2 gap-2">
                    <For each={targets()}>
                      {({ p, i }) => (
                        <button
                          onClick={() =>
                            props.onAction({ type: "Freeze", target: i })
                          }
                          class="py-3 bg-cyan-700 hover:bg-cyan-600 text-white rounded-xl font-bold transition-colors"
                        >
                          P{i}
                          <span class="ml-1 text-cyan-300 text-xs tabular-nums">
                            ({calculateScore(p)} pts)
                          </span>
                        </button>
                      )}
                    </For>
                  </div>
                </div>
              </Show>

              {/* ── Tap3 step 1: choose target ── */}
              <Show when={isPickingTap3Target()}>
                <div class="bg-gray-900 border border-violet-500/30 rounded-2xl p-4">
                  <p class="text-sm text-center text-gray-400 mb-3">
                    👆 Choose a player to{" "}
                    <strong class="text-white">Tap 3</strong>
                  </p>
                  <div class="grid grid-cols-2 gap-2">
                    <For each={targets()}>
                      {({ p, i }) => (
                        <button
                          onClick={() => setTap3Target(i)}
                          class="py-3 bg-violet-700 hover:bg-violet-600 text-white rounded-xl font-bold transition-colors"
                        >
                          P{i}
                          <span class="ml-1 text-violet-300 text-xs tabular-nums">
                            ({calculateScore(p)} pts)
                          </span>
                        </button>
                      )}
                    </For>
                  </div>
                </div>
              </Show>

              {/* ── Tap3 step 2: enter drawn cards ── */}
              <Show when={isCollectingTap3Cards()}>
                <div class="bg-gray-900 border border-violet-500/40 rounded-2xl p-4 space-y-4">
                  <div class="flex items-center justify-between">
                    <div>
                      <p class="text-sm font-bold text-violet-300">
                        👆 Tap 3 — P{tap3Target()} draws
                      </p>
                      <p class="text-xs text-gray-500 mt-0.5">
                        Enter each card P{tap3Target()} actually flipped
                      </p>
                    </div>
                    <span class="text-xs text-gray-500 tabular-nums">
                      {tap3Cards().length} / 3
                    </span>
                  </div>

                  {/* Collected cards so far */}
                  <Show when={tap3Cards().length > 0}>
                    <div class="flex flex-wrap gap-2 items-center">
                      <For each={tap3Cards()}>
                        {(card, idx) => (
                          <div class="relative group">
                            <Show when={card.type === "Number"}>
                              <SmallNumberCard
                                n={
                                  (card as { type: "Number"; value: number })
                                    .value
                                }
                              />
                            </Show>
                            <Show when={card.type !== "Number"}>
                              <div class="bg-indigo-600 w-8 h-10 rounded-md flex items-center justify-center text-white text-[10px] font-bold text-center leading-tight px-0.5">
                                {card.type === "Action"
                                  ? (
                                      card as {
                                        type: "Action";
                                        value: { type: string };
                                      }
                                    ).value.type.slice(0, 3)
                                  : card.type === "Modifier" &&
                                      (
                                        card as {
                                          type: "Modifier";
                                          value: { type: string };
                                        }
                                      ).value.type === "Multiply2"
                                    ? "×2"
                                    : `+${(card as { type: "Modifier"; value: { type: "Add"; value: number } }).value.value}`}
                              </div>
                            </Show>
                            <button
                              onClick={() => removeTap3Card(idx())}
                              class="absolute -top-1.5 -right-1.5 w-4 h-4 bg-red-600 hover:bg-red-500 rounded-full text-white text-[10px] font-bold hidden group-hover:flex items-center justify-center"
                              title="Remove"
                            >
                              ×
                            </button>
                          </div>
                        )}
                      </For>
                    </div>
                  </Show>

                  {/* Status (bust / flip7) */}
                  <Show when={tap3Status() !== null}>
                    <div class="text-center text-sm font-bold text-amber-400 py-1">
                      {tap3Status()}
                    </div>
                  </Show>

                  {/* Card picker (hidden when done) */}
                  <Show when={!tap3Done()}>
                    <div class="border-t border-gray-800 pt-4">
                      <p class="text-xs text-gray-500 mb-3">
                        Pick card #{tap3Cards().length + 1} that P{tap3Target()}{" "}
                        drew:
                      </p>
                      <CardPicker deck={s().deck} onPick={addTap3Card} />
                    </div>
                  </Show>

                  {/* Confirm / cancel */}
                  <div class="grid grid-cols-2 gap-2 pt-1">
                    <button
                      onClick={() => {
                        setTap3Target(null);
                        setTap3Cards([]);
                      }}
                      class="py-2.5 bg-gray-800 hover:bg-gray-700 text-gray-400 rounded-xl font-bold text-sm transition-colors border border-gray-700"
                    >
                      Cancel
                    </button>
                    <button
                      disabled={tap3Cards().length === 0}
                      onClick={confirmTap3}
                      class="py-2.5 bg-violet-600 hover:bg-violet-500 disabled:opacity-40 disabled:cursor-not-allowed text-white rounded-xl font-bold text-sm transition-colors"
                    >
                      Confirm Tap3
                    </button>
                  </div>
                </div>
              </Show>

              {/* ── Normal draw: show card picker ── */}
              <Show when={!pending()}>
                <div class="space-y-3">
                  {/* Card picker drawer */}
                  <div class="bg-gray-900 border border-gray-800 rounded-2xl p-4">
                    <p class="text-xs text-gray-500 mb-3 uppercase tracking-wider">
                      Which card did P{s().current_player} flip?
                    </p>
                    <CardPicker
                      deck={s().deck}
                      onPick={(card) =>
                        props.onAction({ type: "DrawKnown", card })
                      }
                    />
                  </div>

                  <button
                    onClick={() => props.onAction({ type: "Stop" })}
                    class="w-full py-3 bg-gray-800 hover:bg-gray-700 text-gray-300 hover:text-white font-bold rounded-2xl transition-colors border border-gray-700"
                  >
                    ✋ Stop (lock in score)
                  </button>
                </div>
              </Show>
              <div class="flex items-center justify-between mb-4">
                <div>
                  <div class="text-xs text-gray-500 uppercase tracking-wider">
                    Your turn
                  </div>
                  <div class="text-xl font-bold text-amber-400">
                    Player {s().current_player}
                  </div>
                </div>
                <div class="text-right">
                  <div class="text-xs text-gray-500">Score</div>
                  <div class="text-3xl font-black tabular-nums text-white">
                    {cur().status.type === "Busted" ? "BUST" : score()}
                  </div>
                </div>
              </div>

              <div class="flex flex-wrap gap-2 min-h-20 items-center">
                <Show when={cur().numbers.length === 0}>
                  <span class="text-gray-600 text-sm italic">No cards yet</span>
                </Show>
                <For each={cur().numbers}>{(n) => <BigNumberCard n={n} />}</For>
              </div>

              <Show when={cur().modifiers.length > 0 || cur().has_lifeline}>
                <div class="flex flex-wrap gap-2 mt-3 pt-3 border-t border-gray-800">
                  <For each={cur().modifiers}>
                    {(m) => <ModifierTag mod={m} />}
                  </For>
                  <Show when={cur().has_lifeline}>
                    <div class="bg-amber-600/20 border border-amber-600/30 px-2.5 py-1 rounded-md text-amber-400 text-xs font-bold">
                      🛡️ Lifeline
                    </div>
                  </Show>
                </div>
              </Show>
            </div>
          </div>
        </Show>
        {/* Left: all players */}
        <div class="lg:col-span-3">
          <p class="text-xs text-gray-500 uppercase tracking-wider font-medium mb-3">
            Players
          </p>
          <div class="grid grid-cols-2 gap-3">
            <For each={s().players}>
              {(player, i) => (
                <PlayerRow
                  player={player}
                  index={i()}
                  isCurrent={i() === s().current_player}
                  isPlaying={isPlaying()}
                />
              )}
            </For>
          </div>
        </div>
      </div>

      {/* Round-over overlay */}
      <Show when={s().phase.type === "RoundOver"}>
        <RoundOverModal
          state={s()}
          onNextRound={props.onNextRound}
          onEndGame={props.onEndGame}
        />
      </Show>
    </div>
  );
}
