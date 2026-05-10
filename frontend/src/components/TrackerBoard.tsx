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
import { SmallNumberCard } from "./shared/Cards";
import { PlayerRow } from "./shared/PlayerRow";
import { AdvisorPanel } from "./shared/AdvisorPanel";
import { RoundOverModal } from "./shared/RoundOverModal";
import { BoardHeader } from "./shared/BoardHeader";
import { ChainBanner } from "./shared/ChainBanner";
import { CurrentPlayerHand } from "./shared/CurrentPlayerHand";
import { TargetGrid } from "./shared/TargetGrid";
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
      <BoardHeader
        round={s().round}
        drawPileLength={s().deck.draw_pile.length}
        discardPileLength={s().deck.discard_pile.length}
        mode="tracker"
        canUndo={props.canUndo}
        onUndo={props.onUndo}
      />

      {/* ── Main grid ── */}
      <div class="flex-1 max-w-5xl mx-auto w-full p-4 grid lg:grid-cols-5 gap-4 items-start">
        {/* Right: action panel */}
        <Show when={isPlaying()}>
          <div class="lg:col-span-2 space-y-4 top-14">
            {/* Chain-resolution context banner */}
            <Show when={s().current_player !== s().turn_holder}>
              <ChainBanner
                currentPlayer={s().current_player}
                turnHolder={s().turn_holder}
              />
            </Show>

            {/* Advisor — hidden while collecting Tap3 cards to save space */}
            <Show when={props.recommendation && !isCollectingTap3Cards()}>
              {(_) => <AdvisorPanel rec={props.recommendation!} />}
            </Show>

            {/* ── Freeze target picker ── */}
            <Show when={pending()?.type === "Freeze"}>
              <div class="bg-gray-900 border border-cyan-500/30 rounded-2xl p-4">
                <p class="text-sm text-center text-gray-400 mb-3">
                  🧊 Choose a player to{" "}
                  <strong class="text-white">Freeze</strong>
                </p>
                <TargetGrid
                  targets={targets()}
                  variant="cyan"
                  onSelect={(i) =>
                    props.onAction({ type: "Freeze", target: i })
                  }
                />
              </div>
            </Show>

            {/* ── Tap3 step 1: choose target ── */}
            <Show when={isPickingTap3Target()}>
              <div class="bg-gray-900 border border-violet-500/30 rounded-2xl p-4">
                <p class="text-sm text-center text-gray-400 mb-3">
                  👆 Choose a player to{" "}
                  <strong class="text-white">Tap 3</strong>
                </p>
                <TargetGrid
                  targets={targets()}
                  variant="violet"
                  onSelect={(i) => setTap3Target(i)}
                />
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

            {/* Current player hand */}
            <CurrentPlayerHand
              player={s().players[s().current_player]}
              playerIndex={s().current_player}
            />
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
