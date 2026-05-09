import { For, Show } from "solid-js";
import type {
  GameState,
  Player,
  PlayerStatus,
  ModifierCard,
  Recommendation,
  Action,
} from "@app/engine";
import { calculateScore } from "@app/utils/scoring";
import RoundOverModal from "@app/components/RoundOverModal";

// ── Colour map — all class strings must be explicit for Tailwind JIT ──────────

const NUM_BG: Record<number, string> = {
  0: "bg-slate-500",
  1: "bg-red-500",
  2: "bg-orange-500",
  3: "bg-amber-500",
  4: "bg-yellow-500",
  5: "bg-lime-500",
  6: "bg-green-500",
  7: "bg-teal-500",
  8: "bg-cyan-500",
  9: "bg-blue-500",
  10: "bg-indigo-500",
  11: "bg-violet-500",
  12: "bg-purple-500",
};

// ── Inline helper components ──────────────────────────────────────────────────

function SmallCard(props: { n: number }) {
  const bg = () => NUM_BG[props.n] ?? "bg-gray-500";
  return (
    <span
      class={`inline-flex items-center justify-center rounded w-7 h-9 text-sm font-bold text-white shadow-sm select-none ${bg()}`}
    >
      {props.n}
    </span>
  );
}

function BigCard(props: { n: number }) {
  const bg = () => NUM_BG[props.n] ?? "bg-gray-500";
  return (
    <span
      class={`inline-flex items-center justify-center rounded-lg w-12 h-16 text-2xl font-black text-white shadow-lg select-none ${bg()}`}
    >
      {props.n}
    </span>
  );
}

function ModifierTag(props: { mod: ModifierCard }) {
  const label = () => {
    const m = props.mod;
    return m.type === "Add" ? `+${m.value}` : "×2";
  };
  const bg = () =>
    props.mod.type === "Add" ? "bg-purple-600" : "bg-indigo-600";
  return (
    <span
      class={`inline-flex items-center justify-center rounded-full px-2.5 py-0.5 text-xs font-bold text-white shadow-sm ${bg()}`}
    >
      {label()}
    </span>
  );
}

function StatusBadge(props: { status: PlayerStatus }) {
  const config = () => {
    switch (props.status.type) {
      case "Active":
        return { label: "Active", cls: "bg-emerald-700 text-emerald-100" };
      case "Stopped":
        return { label: "Stopped", cls: "bg-blue-700 text-blue-100" };
      case "Busted":
        return { label: "Busted", cls: "bg-red-700 text-red-100" };
      case "Flip7":
        return { label: "FLIP 7!", cls: "bg-amber-500 text-gray-900" };
    }
  };
  return (
    <span
      class={`inline-flex items-center px-2 py-0.5 rounded text-xs font-semibold ${config().cls}`}
    >
      {config().label}
    </span>
  );
}

function PlayerRow(props: {
  player: Player;
  index: number;
  isCurrent: boolean;
  isPlaying: boolean;
}) {
  const highlight = () => props.isCurrent && props.isPlaying;
  const roundScore = () =>
    props.player.status.type === "Busted" ? null : calculateScore(props.player);

  return (
    <div
      class={`rounded-xl p-4 border transition-all ${
        highlight()
          ? "border-amber-400 ring-2 ring-amber-400/30 bg-gray-800"
          : "border-gray-700 bg-gray-800/50"
      }`}
    >
      {/* Header row */}
      <div class="flex items-center gap-2 mb-3 flex-wrap">
        <Show when={highlight()}>
          <span class="text-amber-400 text-sm">▶</span>
        </Show>
        <span class="font-bold text-white text-sm">P{props.index + 1}</span>
        <StatusBadge status={props.player.status} />
        <Show when={props.player.has_lifeline}>
          <span class="text-base" title="Lifeline available">
            🛡️
          </span>
        </Show>
        <div class="ml-auto flex items-center gap-3">
          <div class="text-right">
            <div class="text-xs text-gray-500">Round</div>
            <div class="font-bold text-white text-sm">
              {roundScore() !== null ? roundScore() : "—"}
            </div>
          </div>
          <div class="text-right">
            <div class="text-xs text-gray-500">Total</div>
            <div class="font-bold text-amber-400 text-sm">
              {props.player.total_score}
            </div>
          </div>
        </div>
      </div>

      {/* Cards row */}
      <div class="flex flex-wrap gap-1.5 items-center min-h-9">
        <For each={props.player.numbers}>{(n) => <SmallCard n={n} />}</For>
        <For each={props.player.modifiers}>
          {(mod) => <ModifierTag mod={mod} />}
        </For>
        <Show
          when={
            props.player.numbers.length === 0 &&
            props.player.modifiers.length === 0
          }
        >
          <span class="text-gray-600 text-xs italic">No cards</span>
        </Show>
      </div>
    </div>
  );
}

function BustBar(props: { probability: number }) {
  const pct = () => Math.round(props.probability * 100);
  const barCls = () => {
    const p = props.probability;
    if (p < 0.25) return "bg-green-500";
    if (p < 0.5) return "bg-amber-500";
    return "bg-red-500";
  };
  const labelCls = () => {
    const p = props.probability;
    if (p < 0.25) return "text-green-400";
    if (p < 0.5) return "text-amber-400";
    return "text-red-400";
  };
  return (
    <div class="space-y-1">
      <div class="flex justify-between text-xs">
        <span class="text-gray-400">Bust risk</span>
        <span class={`font-bold ${labelCls()}`}>{pct()}%</span>
      </div>
      <div class="h-2 bg-gray-700 rounded-full overflow-hidden">
        <div
          class={`h-full rounded-full transition-all duration-300 ${barCls()}`}
          style={{ width: `${pct()}%` }}
        />
      </div>
    </div>
  );
}

function AdvisorPanel(props: { rec: Recommendation }) {
  const actionLabel = () => {
    const a = props.rec.action;
    switch (a.type) {
      case "Draw":
        return "🃏 Draw";
      case "Stop":
        return "✋ Stop";
      case "Target":
        return `🎯 Target P${a.index + 1}`;
    }
  };

  const fmt = (n: number | null) => (n !== null ? n.toFixed(1) : "—");

  return (
    <div class="bg-gray-900 border border-gray-700 rounded-xl p-4 space-y-3">
      <div class="text-xs text-gray-500 uppercase tracking-widest font-semibold">
        AI Advisor
      </div>
      <div class="text-lg font-bold text-white">{actionLabel()}</div>
      <div class="grid grid-cols-2 gap-2 text-sm">
        <div class="bg-gray-800 rounded-lg p-2.5">
          <div class="text-gray-500 text-xs mb-0.5">If draw</div>
          <div class="font-bold text-white">
            {fmt(props.rec.expected_score_draw)}
          </div>
        </div>
        <div class="bg-gray-800 rounded-lg p-2.5">
          <div class="text-gray-500 text-xs mb-0.5">If stop</div>
          <div class="font-bold text-white">
            {fmt(props.rec.expected_score_stop)}
          </div>
        </div>
      </div>
      <BustBar probability={props.rec.bust_probability} />
    </div>
  );
}

function TargetPicker(props: {
  state: GameState;
  onAction: (a: Action) => void;
}) {
  const effect = () => props.state.pending_effect!;
  const effectLabel = () =>
    effect().type === "Freeze" ? "Freeze 🧊" : "Tap3 👆";

  return (
    <div class="bg-gray-900 border border-amber-500/40 rounded-xl p-4 space-y-3">
      <div class="text-amber-400 font-bold text-sm">
        Resolve: {effectLabel()}
      </div>
      <div class="text-xs text-gray-400">Choose a target player:</div>
      <div class="flex flex-col gap-2">
        <For each={props.state.players}>
          {(player, i) => (
            <Show
              when={
                i() !== props.state.current_player &&
                player.status.type === "Active"
              }
            >
              <button
                onClick={() => {
                  const eff = effect();
                  props.onAction(
                    eff.type === "Freeze"
                      ? { type: "Freeze", target: i() }
                      : { type: "Tap3", target: i() },
                  );
                }}
                class="w-full py-2.5 px-4 rounded-lg bg-gray-700 hover:bg-gray-600 active:scale-98 text-white font-semibold transition-all text-left text-sm flex items-center justify-between"
              >
                <span>P{i() + 1}</span>
                <span class="text-gray-400 text-xs">
                  Score: {calculateScore(player)}
                </span>
              </button>
            </Show>
          )}
        </For>
      </div>
    </div>
  );
}

// ── GameBoard ─────────────────────────────────────────────────────────────────

interface GameBoardProps {
  state: GameState;
  recommendation: Recommendation | null;
  onAction: (a: Action) => void;
  onNextRound: () => void;
  onEndGame: () => void;
}

export default function GameBoard(props: GameBoardProps) {
  const currentPlayer = () => props.state.players[props.state.current_player];
  const isPlaying = () => props.state.phase.type === "Playing";

  return (
    <div class="min-h-screen bg-gray-950 flex flex-col">
      {/* Sticky header */}
      <header class="sticky top-0 z-20 bg-gray-900/95 backdrop-blur border-b border-gray-800 px-4 py-3">
        <div class="max-w-5xl mx-auto flex items-center justify-between gap-4">
          <div class="text-2xl font-black tracking-tighter shrink-0">
            <span class="text-white">FLIP</span>
            <span class="text-amber-400"> 7</span>
          </div>
          <div class="text-gray-300 font-bold text-sm">
            Round {props.state.round}
          </div>
          <div class="flex gap-2 text-xs ml-auto">
            <span class="bg-gray-800 border border-gray-700 px-3 py-1 rounded-full text-gray-300 font-medium">
              🃏 {props.state.deck.draw_pile.length} left
            </span>
            <span class="bg-gray-800 border border-gray-700 px-3 py-1 rounded-full text-gray-400 font-medium">
              🗂 {props.state.deck.discard_pile.length} discarded
            </span>
          </div>
        </div>
      </header>

      {/* Body */}
      <main class="flex-1 max-w-5xl mx-auto w-full px-4 py-6">
        <div class="lg:grid lg:grid-cols-5 lg:gap-6 lg:items-start">
          {/* ── Left: player list (3 cols) ── */}
          <div class="lg:col-span-3 mb-6 lg:mb-0">
            <h2 class="text-xs text-gray-500 uppercase tracking-widest font-semibold mb-3">
              Players
            </h2>
            <div class="grid grid-cols-2 gap-3">
              <For each={props.state.players}>
                {(player, i) => (
                  <PlayerRow
                    player={player}
                    index={i()}
                    isCurrent={i() === props.state.current_player}
                    isPlaying={isPlaying()}
                  />
                )}
              </For>
            </div>
          </div>

          {/* ── Right: action panel (2 cols) — hidden when RoundOver ── */}
          <Show when={isPlaying()}>
            <div class="lg:col-span-2 space-y-4 sticky top-14">
              {/* Current player hand */}
              <div class="bg-gray-800 border border-gray-700 rounded-xl p-4 space-y-3">
                <div class="flex items-center gap-2">
                  <span class="text-xs text-gray-400 uppercase tracking-widest font-semibold">
                    Your Hand
                  </span>
                  <span class="text-gray-500 font-bold text-sm ml-auto">
                    P{props.state.current_player + 1}
                  </span>
                  <Show when={currentPlayer().has_lifeline}>
                    <span class="text-base" title="Lifeline available">
                      🛡️
                    </span>
                  </Show>
                </div>

                {/* Big card display */}
                <div class="flex flex-wrap gap-2 min-h-18 items-start">
                  <For each={currentPlayer().numbers}>
                    {(n) => <BigCard n={n} />}
                  </For>
                  <For each={currentPlayer().modifiers}>
                    {(mod) => <ModifierTag mod={mod} />}
                  </For>
                  <Show
                    when={
                      currentPlayer().numbers.length === 0 &&
                      currentPlayer().modifiers.length === 0
                    }
                  >
                    <span class="text-gray-600 text-sm italic self-center">
                      No cards yet
                    </span>
                  </Show>
                </div>

                {/* Score */}
                <div class="flex justify-between items-center pt-2 border-t border-gray-700">
                  <span class="text-gray-400 text-sm">Round Score</span>
                  <span class="text-3xl font-black text-white tabular-nums">
                    {calculateScore(currentPlayer())}
                  </span>
                </div>
              </div>

              {/* AI Advisor panel */}
              <Show when={props.recommendation !== null}>
                <AdvisorPanel rec={props.recommendation!} />
              </Show>

              {/* Action buttons or TargetPicker */}
              <Show
                when={props.state.pending_effect !== null}
                fallback={
                  <div class="flex gap-3">
                    <button
                      onClick={() => props.onAction({ type: "Draw" })}
                      class="flex-1 py-3.5 rounded-xl bg-amber-500 hover:bg-amber-400 active:scale-98 text-gray-900 font-bold text-lg transition-all shadow-lg shadow-amber-900/30"
                    >
                      🃏 Draw
                    </button>
                    <button
                      onClick={() => props.onAction({ type: "Stop" })}
                      class="flex-1 py-3.5 rounded-xl bg-blue-600 hover:bg-blue-500 active:scale-98 text-white font-bold text-lg transition-all shadow-lg shadow-blue-900/30"
                    >
                      ✋ Stop
                    </button>
                  </div>
                }
              >
                <TargetPicker state={props.state} onAction={props.onAction} />
              </Show>
            </div>
          </Show>
        </div>
      </main>

      {/* Round-over overlay */}
      <Show when={props.state.phase.type === "RoundOver"}>
        <RoundOverModal
          state={props.state}
          onNextRound={props.onNextRound}
          onEndGame={props.onEndGame}
        />
      </Show>
    </div>
  );
}
