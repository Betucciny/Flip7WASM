import { Show } from "solid-js";

export type BoardMode = "simulator" | "tracker";

interface Props {
  round: number;
  drawPileLength: number;
  discardPileLength: number;
  mode: BoardMode;
  canUndo: boolean;
  onUndo: () => void;
}

export function BoardHeader(props: Props) {
  return (
    <header class="bg-gray-900 border-b border-gray-800 sticky top-0 z-10">
      <div class="max-w-5xl mx-auto px-4 py-3 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <span class="font-black text-lg">
            <span class="text-white">FLIP</span>
            <span class="text-amber-400"> 7</span>
          </span>
          <span class="text-gray-600">·</span>
          <span class="text-gray-400 text-sm">Round {props.round}</span>
          <Show when={props.mode === "simulator"}>
            <span class="bg-gray-800 text-gray-400 text-xs px-2 py-0.5 rounded-full border border-gray-700">
              🎮 Simulator
            </span>
          </Show>
          <Show when={props.mode === "tracker"}>
            <span class="bg-blue-900/50 text-blue-400 text-xs px-2 py-0.5 rounded-full border border-blue-700/50">
              📋 Tracker
            </span>
          </Show>
        </div>
        <div class="flex items-center gap-3">
          <div class="flex gap-3 text-xs text-gray-500">
            <span>🃏 {props.drawPileLength} left</span>
            <span>🗑 {props.discardPileLength} used</span>
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
  );
}
