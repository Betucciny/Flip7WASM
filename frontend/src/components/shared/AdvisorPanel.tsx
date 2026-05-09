import { Show } from "solid-js";
import type { Recommendation } from "@app/engine";

// ── Bust probability bar ──────────────────────────────────────────────────────

export function BustBar(props: { probability: number }) {
  const pct = () => Math.round(props.probability * 100);
  const barColor = () => {
    if (pct() > 50) return "bg-red-500";
    if (pct() > 25) return "bg-amber-500";
    return "bg-emerald-500";
  };
  const textColor = () => {
    if (pct() > 50) return "text-red-400 font-bold";
    if (pct() > 25) return "text-amber-400 font-bold";
    return "text-emerald-400";
  };

  return (
    <div>
      <div class="flex justify-between text-xs text-gray-400 mb-1.5">
        <span>Bust risk on next draw</span>
        <span class={textColor()}>{pct()}%</span>
      </div>
      <div class="h-2 bg-gray-800 rounded-full overflow-hidden">
        <div
          class={`h-full rounded-full transition-all duration-500 ${barColor()}`}
          style={{ width: `${Math.min(100, pct())}%` }}
        />
      </div>
    </div>
  );
}

// ── Advisor panel ─────────────────────────────────────────────────────────────

export function AdvisorPanel(props: { rec: Recommendation }) {
  const rec = () => props.rec;
  const action = () => rec().action;

  return (
    <div class="bg-gray-900 border border-gray-700 rounded-2xl p-4 space-y-3">
      <div class="text-xs text-gray-500 font-medium uppercase tracking-wider">
        AI Advisor
      </div>

      {/* Recommendation */}
      <div class="flex items-center gap-2">
        <Show when={action().type === "Draw"}>
          <span class="text-xl">🃏</span>
          <span class="text-emerald-400 font-bold text-lg">Draw</span>
        </Show>
        <Show when={action().type === "Stop"}>
          <span class="text-xl">✋</span>
          <span class="text-blue-400 font-bold text-lg">Stop</span>
        </Show>
        <Show when={action().type === "Target"}>
          <span class="text-xl">🎯</span>
          <span class="text-violet-400 font-bold text-lg">
            {/* TypeScript narrowing via Show guard above; cast is safe here */}
            Target P{(action() as { type: "Target"; index: number }).index}
          </span>
        </Show>
      </div>

      {/* Expected scores grid */}
      <Show
        when={
          rec().expected_score_draw !== null ||
          rec().expected_score_stop !== null
        }
      >
        <div class="grid grid-cols-2 gap-2 text-xs">
          <div class="bg-emerald-500/10 border border-emerald-500/20 rounded-lg p-2 text-center">
            <div class="text-gray-500 mb-0.5">If draw</div>
            <div class="text-emerald-400 font-bold text-base tabular-nums">
              {rec().expected_score_draw?.toFixed(1) ?? "—"}
            </div>
          </div>
          <div class="bg-blue-500/10 border border-blue-500/20 rounded-lg p-2 text-center">
            <div class="text-gray-500 mb-0.5">If stop</div>
            <div class="text-blue-400 font-bold text-base tabular-nums">
              {rec().expected_score_stop?.toFixed(1) ?? "—"}
            </div>
          </div>
        </div>
      </Show>

      <BustBar probability={rec().bust_probability} />
    </div>
  );
}
