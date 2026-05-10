/**
 * SimulatorBoard — game board for Simulator mode.
 *
 * The engine draws random cards from its internal shuffled deck.
 * Players interact with Draw / Stop buttons; Freeze and Tap3 use target pickers.
 */
import { For, Show } from "solid-js";
import type { Action, GameState, Recommendation } from "@app/engine";
import { PlayerRow } from "./shared/PlayerRow";
import { AdvisorPanel } from "./shared/AdvisorPanel";
import { RoundOverModal } from "./shared/RoundOverModal";
import { BoardHeader } from "./shared/BoardHeader";
import { ChainBanner } from "./shared/ChainBanner";
import { CurrentPlayerHand } from "./shared/CurrentPlayerHand";
import { TargetGrid } from "./shared/TargetGrid";

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
  const pending = () => s().pending_effect;
  const isPlaying = () => s().phase.type === "Playing";

  /** Other active players — valid Freeze / Tap3 targets. */
  const targets = () =>
    s()
      .players.map((p, i) => ({ p, i }))
      .filter(({ p }) => p.status.type === "Active");

  function fire(action: Action) {
    props.onAction(action);
  }

  return (
    <div class="min-h-screen bg-gray-950 flex flex-col">
      {/* ── Header ── */}
      <BoardHeader
        round={s().round}
        drawPileLength={s().deck.draw_pile.length}
        discardPileLength={s().deck.discard_pile.length}
        mode="simulator"
        canUndo={props.canUndo}
        onUndo={props.onUndo}
      />

      {/* ── Main grid ── */}
      <div class="flex-1 max-w-7xl mx-auto w-full p-4 grid lg:grid-cols-5 gap-4 items-start">
        <Show when={isPlaying()}>
          <div class="lg:col-span-2 space-y-4">
            {/* Chain-resolution context banner */}
            <Show when={s().current_player !== s().turn_holder}>
              <ChainBanner
                currentPlayer={s().current_player}
                turnHolder={s().turn_holder}
              />
            </Show>

            {/* Advisor */}
            <Show when={props.recommendation}>
              {(rec) => <AdvisorPanel rec={rec()} />}
            </Show>

            {/* Actions */}
            <Show when={pending()}>
              {(eff) => (
                <div class="bg-gray-900 border border-violet-500/30 rounded-2xl p-4">
                  <p class="text-sm text-gray-400 mb-3 text-center">
                    {eff().type === "Freeze"
                      ? "🧊 Choose a player to Freeze"
                      : "👆 Choose a player to Tap 3 cards"}
                  </p>
                  <TargetGrid
                    targets={targets()}
                    variant="violet"
                    onSelect={(i) =>
                      fire(
                        eff().type === "Freeze"
                          ? { type: "Freeze", target: i }
                          : { type: "Tap3", target: i },
                      )
                    }
                  />
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
            <CurrentPlayerHand
              player={s().players[s().current_player]}
              playerIndex={s().current_player}
            />
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
