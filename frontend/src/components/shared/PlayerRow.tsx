import { For, Show } from "solid-js";
import type { Player } from "@app/engine";
import { calculateScore } from "../../utils/scoring";
import { SmallNumberCard, ModifierTag } from "./Cards";
import { StatusBadge } from "./StatusBadge";

interface Props {
  player: Player;
  index: number;
  /** Whether this is the current-turn player. */
  isCurrent: boolean;
  /** Whether the game phase is Playing (suppresses arrow during RoundOver). */
  isPlaying: boolean;
}

export function PlayerRow(props: Props) {
  const active = () => props.isCurrent && props.isPlaying;
  const score = () => calculateScore(props.player);

  return (
    <div
      class={`p-3 rounded-2xl border transition-all ${
        active()
          ? "bg-gray-800 border-amber-500/50 ring-1 ring-amber-500/20"
          : "bg-gray-900 border-gray-800"
      }`}
    >
      {/* Header row */}
      <div class="flex items-center justify-between mb-3">
        <div class="flex items-center gap-2 flex-wrap">
          <span
            class={`w-4 text-amber-400 text-sm ${active() ? "" : "invisible"}`}
          >
            ▶
          </span>
          <span
            class={`font-bold ${active() ? "text-amber-400" : "text-gray-300"}`}
          >
            P{props.index}
          </span>
          <StatusBadge status={props.player.status} />
          <Show when={props.player.has_lifeline}>
            <span title="Has Lifeline" class="text-base leading-none">
              🛡️
            </span>
          </Show>
        </div>

        <div class="text-right shrink-0 ml-2">
          <div class="text-xs text-gray-500">this round</div>
          <div
            class={`text-lg font-bold tabular-nums ${active() ? "text-amber-400" : "text-gray-200"}`}
          >
            {props.player.status.type === "Busted" ? "—" : score()}
          </div>
          <div class="text-xs text-gray-500 tabular-nums">
            total: {props.player.total_score}
          </div>
        </div>
      </div>

      {/* Cards row */}
      <div class="flex flex-wrap gap-1 items-center min-h-8">
        <Show
          when={
            props.player.numbers.length === 0 &&
            props.player.modifiers.length === 0
          }
        >
          <span class="text-gray-600 text-sm italic">No cards yet</span>
        </Show>
        <For each={props.player.numbers}>
          {(n) => <SmallNumberCard n={n} />}
        </For>
        <For each={props.player.modifiers}>
          {(m) => <ModifierTag mod={m} />}
        </For>
      </div>
    </div>
  );
}
