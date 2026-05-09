import { For } from "solid-js";
import type { GameState } from "@app/engine";

interface Props {
  state: GameState;
  onRestart: () => void;
}

export default function GameOverScreen(props: Props) {
  const sorted = () =>
    props.state.players
      .map((p, i) => ({ p, i }))
      .sort((a, b) => b.p.total_score - a.p.total_score);

  return (
    <div class="min-h-screen bg-gray-950 flex items-center justify-center p-4">
      <div class="w-full max-w-md">
        <div class="bg-gray-900 border border-gray-800 rounded-3xl p-8 shadow-2xl text-center">
          <div class="text-6xl mb-4">🏆</div>
          <h1 class="text-3xl font-black text-white mb-1">Game Over!</h1>
          <p class="text-gray-500 text-sm mb-8">Final Scores</p>

          <div class="space-y-3 mb-8">
            <For each={sorted()}>
              {(item, rank) => (
                <div
                  class={`flex items-center justify-between p-4 rounded-2xl ${
                    rank() === 0
                      ? "bg-amber-500/15 border border-amber-500/30"
                      : "bg-gray-800"
                  }`}
                >
                  <div class="flex items-center gap-3">
                    <span class="text-xl font-black text-gray-600 w-7 text-center tabular-nums">
                      {rank() + 1}.
                    </span>
                    {rank() === 0 && <span class="text-xl">👑</span>}
                    <span
                      class={`text-lg font-bold ${
                        rank() === 0 ? "text-amber-400" : "text-gray-200"
                      }`}
                    >
                      Player {item.i}
                    </span>
                  </div>
                  <span
                    class={`text-2xl font-black tabular-nums ${
                      rank() === 0 ? "text-amber-400" : "text-gray-300"
                    }`}
                  >
                    {item.p.total_score}
                    <span class="text-sm font-normal text-gray-500 ml-1">
                      pts
                    </span>
                  </span>
                </div>
              )}
            </For>
          </div>

          <button
            onClick={props.onRestart}
            class="w-full py-4 bg-emerald-600 hover:bg-emerald-500 text-white text-xl font-bold rounded-2xl transition-colors shadow-lg shadow-emerald-900/40"
          >
            Play Again
          </button>
        </div>
      </div>
    </div>
  );
}
