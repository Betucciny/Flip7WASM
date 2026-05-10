/**
 * SimulatorBoard — game board for Simulator mode.
 *
 * The engine draws random cards from its internal shuffled deck.
 * Players interact with Draw / Stop buttons; Freeze and Tap3 use target pickers.
 */
import { For, Show } from "solid-js";
import type { Action, GameState, Recommendation } from "@app/engine";
import { calculateScore } from "../utils/scoring";
import { BigNumberCard, ModifierTag } from "./shared/Cards";
import { PlayerRow } from "./shared/PlayerRow";
import { AdvisorPanel } from "./shared/AdvisorPanel";
import { RoundOverModal } from "./shared/RoundOverModal";

interface Props {
  state: GameState;
  recommendation: Recommendation | null;
  onAction: (action: Action) => void;
  onNextRound: () => void;
  onEndGame: () => void;
  canUndo: boolean;
  onUndo: () => void;
}

export default function SimulatorBoard(props: Props) {
  const s = () => props.state;
  const cur = () => s().players[s().current_player];
  const score = () => calculateScore(cur());
  const pending = () => s().pending_effect;
  const isPlaying = () => s().phase.type === "Playing";

  /** Other active players — valid Freeze / Tap3 targets. */
  const targets = () =>
    s()
      .players.map((p, i) => ({ p, i }))
      .filter(({ p, i }) => p.status.type === "Active");

  function fire(action: Action) {
    props.onAction(action);
  }

  return (
    <div class="min-h-screen bg-gray-950 flex flex-col">
      {/* ── Header ── */}
      <header class="bg-gray-900 border-b border-gray-800 sticky top-0 z-10">
        <div class="max-w-5xl mx-auto px-4 py-3 flex items-center justify-between">
          <div class="flex items-center gap-3">
            <span class="font-black text-lg">
              <span class="text-white">FLIP</span>
              <span class="text-amber-400"> 7</span>
            </span>
            <span class="text-gray-600">·</span>
            <span class="text-gray-400 text-sm">Round {s().round}</span>
            <span class="bg-gray-800 text-gray-400 text-xs px-2 py-0.5 rounded-full border border-gray-700">
              🎮 Simulator
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
      <div class="flex-1 max-w-7xl mx-auto w-full p-4 grid lg:grid-cols-5 gap-4 items-start">
        <Show when={isPlaying()}>
          <div class="lg:col-span-2 space-y-4">
            {/* Chain-resolution context banner */}
            <Show when={s().current_player !== s().turn_holder}>
              <div class="bg-violet-900/30 border border-violet-500/40 rounded-xl px-3 py-2 text-xs text-violet-300">
                ⛓️ <strong>Chain:</strong> P{s().turn_holder}'s Tap3 → P
                {s().current_player} resolves a queued effect
              </div>
            </Show>

            {/* Advisor */}
            <Show when={props.recommendation}>
              {(rec) => <AdvisorPanel rec={rec()} />}
            </Show>

            {/* Actions */}
            <Show when={pending()}>
              {(eff) => (
                /* eff() is the live PendingEffect object — always non-nullish here */
                <div class="bg-gray-900 border border-violet-500/30 rounded-2xl p-4">
                  <p class="text-sm text-gray-400 mb-3 text-center">
                    {eff().type === "Freeze"
                      ? "🧊 Choose a player to Freeze"
                      : "👆 Choose a player to Tap 3 cards"}
                  </p>
                  <div class="grid grid-cols-2 gap-2">
                    <For each={targets()}>
                      {({ p, i }) => (
                        <button
                          onClick={() =>
                            fire(
                              eff().type === "Freeze"
                                ? { type: "Freeze", target: i }
                                : { type: "Tap3", target: i },
                            )
                          }
                          class="py-3 bg-violet-700 hover:bg-violet-600 active:bg-violet-800 text-white rounded-xl font-bold transition-colors"
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
              )}
            </Show>

            <Show when={!pending()}>
              {/* ── Draw / Stop ── */}
              <div class="space-y-3">
                <button
                  onClick={() => fire({ type: "Draw" })}
                  class="w-full py-4 bg-emerald-600 hover:bg-emerald-500 active:bg-emerald-700 text-white text-lg font-bold rounded-2xl transition-colors shadow-lg shadow-emerald-900/40"
                >
                  🃏 Draw
                </button>
                <button
                  onClick={() => fire({ type: "Stop" })}
                  class="w-full py-3 bg-gray-800 hover:bg-gray-700 text-gray-300 hover:text-white font-bold rounded-2xl transition-colors border border-gray-700"
                >
                  ✋ Stop
                </button>
              </div>
            </Show>
            {/* Current player hand */}
            <div class="bg-gray-900 border border-gray-800 rounded-2xl p-4">
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

              {/* Big number cards */}
              <div class="flex flex-wrap gap-2 min-h-20 items-center">
                <Show when={cur().numbers.length === 0}>
                  <span class="text-gray-600 text-sm italic">No cards yet</span>
                </Show>
                <For each={cur().numbers}>{(n) => <BigNumberCard n={n} />}</For>
              </div>

              {/* Modifiers + lifeline */}
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
        <div class="lg:col-span-3 space-y-3">
          <p class="text-xs text-gray-500 uppercase tracking-wider font-medium">
            Players
          </p>
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
