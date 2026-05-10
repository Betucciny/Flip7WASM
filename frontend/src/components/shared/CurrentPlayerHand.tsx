import { For, Show } from "solid-js";
import type { Player } from "@app/engine";
import { calculateScore } from "../../utils/scoring";
import { BigNumberCard, ModifierTag } from "./Cards";

interface Props {
  player: Player;
  playerIndex: number;
}

export function CurrentPlayerHand(props: Props) {
  const score = () => calculateScore(props.player);

  return (
    <div class="bg-gray-900 border border-gray-800 rounded-2xl p-4">
      <div class="flex items-center justify-between mb-4">
        <div>
          <div class="text-xs text-gray-500 uppercase tracking-wider">
            Your turn
          </div>
          <div class="text-xl font-bold text-amber-400">
            Player {props.playerIndex}
          </div>
        </div>
        <div class="text-right">
          <div class="text-xs text-gray-500">Score</div>
          <div class="text-3xl font-black tabular-nums text-white">
            {props.player.status.type === "Busted" ? "BUST" : score()}
          </div>
        </div>
      </div>

      {/* Big number cards */}
      <div class="flex flex-wrap gap-2 min-h-20 items-center">
        <Show when={props.player.numbers.length === 0}>
          <span class="text-gray-600 text-sm italic">No cards yet</span>
        </Show>
        <For each={props.player.numbers}>{(n) => <BigNumberCard n={n} />}</For>
      </div>

      {/* Modifiers + lifeline */}
      <Show when={props.player.modifiers.length > 0 || props.player.has_lifeline}>
        <div class="flex flex-wrap gap-2 mt-3 pt-3 border-t border-gray-800">
          <For each={props.player.modifiers}>
            {(m) => <ModifierTag mod={m} />}
          </For>
          <Show when={props.player.has_lifeline}>
            <div class="bg-amber-600/20 border border-amber-600/30 px-2.5 py-1 rounded-md text-amber-400 text-xs font-bold">
              🛡️ Lifeline
            </div>
          </Show>
        </div>
      </Show>
    </div>
  );
}
