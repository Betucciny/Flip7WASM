import { For } from "solid-js";
import type { Player } from "@app/engine";
import { calculateScore } from "../../utils/scoring";

export interface TargetEntry {
  p: Player;
  i: number;
}

export type TargetVariant = "violet" | "cyan";

const COLOR: Record<TargetVariant, { button: string; score: string }> = {
  violet: {
    button:
      "bg-violet-700 hover:bg-violet-600 active:bg-violet-800",
    score: "text-violet-300",
  },
  cyan: {
    button: "bg-cyan-700 hover:bg-cyan-600",
    score: "text-cyan-300",
  },
};

interface Props {
  targets: TargetEntry[];
  variant: TargetVariant;
  onSelect: (index: number) => void;
}

export function TargetGrid(props: Props) {
  const cls = () => COLOR[props.variant];

  return (
    <div class="grid grid-cols-2 gap-2">
      <For each={props.targets}>
        {({ p, i }) => (
          <button
            onClick={() => props.onSelect(i)}
            class={`py-3 ${cls().button} text-white rounded-xl font-bold transition-colors`}
          >
            P{i}
            <span class={`ml-1 ${cls().score} text-xs tabular-nums`}>
              ({calculateScore(p)} pts)
            </span>
          </button>
        )}
      </For>
    </div>
  );
}
