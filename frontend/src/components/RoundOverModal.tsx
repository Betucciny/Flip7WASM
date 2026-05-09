import { For } from "solid-js";
import type { GameState } from "@app/engine";
import { calculateScore } from "@app/utils/scoring";

interface RoundOverModalProps {
  state: GameState;
  onNextRound: () => void;
  onEndGame: () => void;
}

export default function RoundOverModal(props: RoundOverModalProps) {
  /** Sorted player entries: highest round score first. */
  const playerScores = () =>
    props.state.players
      .map((p, i) => ({
        index: i,
        player: p,
        roundScore: calculateScore(p), // returns 0 for busted
      }))
      .sort((a, b) => b.roundScore - a.roundScore);

  /** The best round score (used to identify the winner). */
  const maxScore = () => {
    const scores = playerScores();
    return scores.length > 0 ? scores[0].roundScore : 0;
  };

  return (
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-sm">
      <div class="bg-gray-900 border border-gray-700 rounded-2xl p-8 w-full max-w-md mx-4 shadow-2xl">
        {/* Header */}
        <div class="text-center mb-6">
          <div class="text-5xl mb-2">🃏</div>
          <h2 class="text-2xl font-black text-white">
            Round {props.state.round} Complete!
          </h2>
        </div>

        {/* Player score list */}
        <div class="space-y-2 mb-6">
          <For each={playerScores()}>
            {(entry) => {
              const isWinner =
                entry.roundScore === maxScore() && entry.roundScore > 0;
              const statusTag =
                entry.player.status.type === "Busted"
                  ? { label: "BUST", cls: "bg-red-700 text-red-100" }
                  : entry.player.status.type === "Flip7"
                    ? { label: "FLIP7", cls: "bg-amber-500 text-gray-900" }
                    : { label: "OK", cls: "bg-gray-600 text-gray-200" };

              return (
                <div
                  class={`flex items-center gap-3 rounded-xl p-3 ${
                    isWinner
                      ? "bg-amber-500/10 border border-amber-500/30"
                      : "bg-gray-800 border border-transparent"
                  }`}
                >
                  <span class="font-bold text-gray-300 text-sm w-8 shrink-0">
                    P{entry.index + 1}
                  </span>
                  {isWinner ? <span class="text-lg">👑</span> : null}
                  <span
                    class={`inline-flex items-center px-2 py-0.5 rounded text-xs font-semibold ${statusTag.cls}`}
                  >
                    {statusTag.label}
                  </span>
                  <span
                    class={`font-bold ml-auto ${isWinner ? "text-amber-400" : "text-green-400"}`}
                  >
                    +{entry.roundScore}
                  </span>
                  <span class="text-gray-400 text-sm shrink-0">
                    → {entry.player.total_score + entry.roundScore} total
                  </span>
                </div>
              );
            }}
          </For>
        </div>

        {/* Action buttons */}
        <div class="flex gap-3">
          <button
            onClick={props.onEndGame}
            class="flex-1 py-3 rounded-xl bg-gray-700 hover:bg-gray-600 active:scale-98 text-white font-bold transition-all"
          >
            End Game
          </button>
          <button
            onClick={props.onNextRound}
            class="flex-1 py-3 rounded-xl bg-green-600 hover:bg-green-500 active:scale-98 text-white font-bold transition-all"
          >
            Next Round →
          </button>
        </div>
      </div>
    </div>
  );
}
