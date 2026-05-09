import { For } from "solid-js";
import type { GameState } from "@app/engine";
import { roundScore } from "../../utils/scoring";

interface Props {
  state: GameState;
  onNextRound: () => void;
  onEndGame: () => void;
}

export function RoundOverModal(props: Props) {
  const s = () => props.state;

  /** Players sorted by this-round contribution (desc). */
  const results = () =>
    s()
      .players.map((p, i) => ({
        index: i,
        player: p,
        thisRound: roundScore(p),
        newTotal: p.total_score + roundScore(p),
      }))
      .sort((a, b) => b.thisRound - a.thisRound);

  return (
    <div class="fixed inset-0 bg-black/75 flex items-center justify-center z-50 p-4 backdrop-blur-sm">
      <div class="bg-gray-900 border border-gray-700 rounded-3xl p-8 max-w-md w-full shadow-2xl">
        {/* Header */}
        <div class="text-center mb-6">
          <div class="text-4xl mb-2">🃏</div>
          <h2 class="text-2xl font-black text-white">
            Round {s().round} Complete!
          </h2>
          <p class="text-gray-500 text-sm mt-1">
            First to 200 total points wins.
          </p>
        </div>

        {/* Scores */}
        <div class="space-y-2 mb-6">
          <For each={results()}>
            {(r, rank) => (
              <div
                class={`flex items-center justify-between p-3 rounded-xl ${
                  rank() === 0
                    ? "bg-amber-500/10 border border-amber-500/20"
                    : "bg-gray-800"
                }`}
              >
                <div class="flex items-center gap-2">
                  {rank() === 0 && <span class="text-amber-400">👑</span>}
                  <span
                    class={`font-bold ${
                      r.player.status.type === "Flip7"
                        ? "text-amber-400"
                        : r.player.status.type === "Busted"
                          ? "text-red-400"
                          : "text-gray-200"
                    }`}
                  >
                    P{r.index}
                  </span>
                  <span
                    class={`text-xs px-2 py-0.5 rounded-full font-medium ${
                      r.player.status.type === "Busted"
                        ? "bg-red-500/20 text-red-400"
                        : r.player.status.type === "Flip7"
                          ? "bg-amber-500/20 text-amber-400"
                          : r.player.status.type === "Stopped"
                            ? "bg-blue-500/20 text-blue-400"
                            : "bg-gray-700 text-gray-400"
                    }`}
                  >
                    {r.player.status.type === "Busted"
                      ? "BUST"
                      : r.player.status.type === "Flip7"
                        ? "FLIP 7!"
                        : "OK"}
                  </span>
                </div>

                <div class="text-right tabular-nums">
                  <div
                    class={`font-bold ${r.thisRound > 0 ? "text-emerald-400" : "text-gray-500"}`}
                  >
                    +{r.thisRound}
                  </div>
                  <div class="text-xs text-gray-500">→ {r.newTotal} total</div>
                </div>
              </div>
            )}
          </For>
        </div>

        {/* Buttons */}
        <div class="grid grid-cols-2 gap-3">
          <button
            onClick={props.onEndGame}
            class="py-3 bg-gray-800 hover:bg-gray-700 text-gray-300 font-bold rounded-2xl transition-colors border border-gray-700"
          >
            End Game
          </button>
          <button
            onClick={props.onNextRound}
            class="py-3 bg-emerald-600 hover:bg-emerald-500 text-white font-bold rounded-2xl transition-colors shadow-lg shadow-emerald-900/40"
          >
            Next Round →
          </button>
        </div>
      </div>
    </div>
  );
}
