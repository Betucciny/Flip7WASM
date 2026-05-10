import { createSignal, Show } from "solid-js";

export type GameMode = "simulator" | "tracker";

interface Props {
  onStart: (
    mode: GameMode,
    playerCount: number,
    simulations: number,
    showHints: boolean,
  ) => void;
}

export default function StartScreen(props: Props) {
  const [count, setCount] = createSignal(3);
  const [mode, setMode] = createSignal<GameMode>("simulator");
  const [simulations, setSimulations] = createSignal(500);
  const [showHints, setShowHints] = createSignal(true);

  const simQuality = () => {
    const n = simulations();
    if (n <= 1000) return { label: "Fast", color: "text-emerald-400" };
    if (n <= 3000) return { label: "Balanced", color: "text-amber-400" };
    if (n <= 6000) return { label: "Accurate", color: "text-blue-400" };
    return { label: "Precise", color: "text-violet-400" };
  };

  return (
    <div class="min-h-screen bg-gray-950 flex items-center justify-center p-4 relative overflow-hidden">
      {/* Decorative background 7s */}
      <div
        class="absolute inset-0 pointer-events-none select-none"
        aria-hidden="true"
      >
        <span class="absolute top-16 left-8 text-[10rem] font-black text-amber-400/5 rotate-12">
          7
        </span>
        <span class="absolute bottom-16 right-8 text-[10rem] font-black text-emerald-400/5 -rotate-12">
          7
        </span>
        <span class="absolute top-1/2 left-1/3 text-[8rem] font-black text-blue-400/5 rotate-45">
          7
        </span>
      </div>

      <div class="relative z-10 w-full max-w-md">
        <div class="bg-gray-900 rounded-3xl p-10 shadow-2xl border border-gray-800">
          {/* Logo */}
          <div class="text-center mb-10">
            <div class="inline-flex items-center justify-center gap-3 mb-4">
              <div class="w-14 h-20 bg-white rounded-xl flex items-center justify-center shadow-lg">
                <span class="text-4xl font-black text-amber-500">7</span>
              </div>
            </div>
            <h1 class="text-5xl font-black tracking-tight leading-none">
              <span class="text-white">FLIP</span>
              <span class="text-amber-400"> 7</span>
            </h1>
            <p class="text-gray-500 text-xs mt-2 tracking-widest uppercase">
              Card Game · First to 200 points wins
            </p>
          </div>

          {/* Mode selector */}
          <div class="mb-8">
            <p class="text-gray-400 text-xs font-medium uppercase tracking-wider text-center mb-3">
              Game Mode
            </p>
            <div class="grid grid-cols-2 gap-2">
              <button
                onClick={() => setMode("simulator")}
                class={`p-4 rounded-2xl border text-left transition-all ${
                  mode() === "simulator"
                    ? "bg-emerald-600/20 border-emerald-500/50 ring-1 ring-emerald-500/30"
                    : "bg-gray-800 border-gray-700 hover:border-gray-600"
                }`}
              >
                <div class="text-lg mb-1">🎮</div>
                <div class="font-bold text-sm text-white">Simulator</div>
                <div class="text-xs text-gray-500 mt-0.5">
                  Random draws · Play online
                </div>
              </button>

              <button
                onClick={() => setMode("tracker")}
                class={`p-4 rounded-2xl border text-left transition-all ${
                  mode() === "tracker"
                    ? "bg-blue-600/20 border-blue-500/50 ring-1 ring-blue-500/30"
                    : "bg-gray-800 border-gray-700 hover:border-gray-600"
                }`}
              >
                <div class="text-lg mb-1">📋</div>
                <div class="font-bold text-sm text-white">Tracker</div>
                <div class="text-xs text-gray-500 mt-0.5">
                  Enter real cards · Track physical game
                </div>
              </button>
            </div>
          </div>

          {/* Player count */}
          <div class="mb-8 text-center">
            <p class="text-gray-400 text-xs font-medium uppercase tracking-wider mb-4">
              Players
            </p>
            <div class="flex items-center justify-center gap-8">
              <button
                onClick={() => setCount((c) => Math.max(2, c - 1))}
                disabled={count() <= 2}
                class="w-14 h-14 rounded-full bg-gray-800 hover:bg-gray-700 disabled:opacity-30 disabled:cursor-not-allowed text-white text-3xl font-light flex items-center justify-center transition-colors border border-gray-700"
              >
                −
              </button>
              <span class="text-6xl font-black text-amber-400 tabular-nums w-16 text-center">
                {count()}
              </span>
              <button
                onClick={() => setCount((c) => Math.min(15, c + 1))}
                disabled={count() >= 15}
                class="w-14 h-14 rounded-full bg-gray-800 hover:bg-gray-700 disabled:opacity-30 disabled:cursor-not-allowed text-white text-3xl font-light flex items-center justify-center transition-colors border border-gray-700"
              >
                +
              </button>
            </div>
          </div>

          {/* AI Hints toggle */}
          <div class="mb-8">
            <p class="text-gray-400 text-xs font-medium uppercase tracking-wider text-center mb-3">
              AI Hints
            </p>
            <div class="grid grid-cols-2 gap-2">
              <button
                onClick={() => setShowHints(true)}
                class={`p-3 rounded-2xl border text-center transition-all ${
                  showHints()
                    ? "bg-violet-600/20 border-violet-500/50 ring-1 ring-violet-500/30"
                    : "bg-gray-800 border-gray-700 hover:border-gray-600"
                }`}
              >
                <div class="text-lg mb-1">✨</div>
                <div class="font-bold text-sm text-white">Hints On</div>
                <div class="text-xs text-gray-500 mt-0.5">
                  AI advisor enabled
                </div>
              </button>
              <button
                onClick={() => setShowHints(false)}
                class={`p-3 rounded-2xl border text-center transition-all ${
                  !showHints()
                    ? "bg-gray-600/20 border-gray-500/50 ring-1 ring-gray-500/30"
                    : "bg-gray-800 border-gray-700 hover:border-gray-600"
                }`}
              >
                <div class="text-lg mb-1">🔕</div>
                <div class="font-bold text-sm text-white">Hints Off</div>
                <div class="text-xs text-gray-500 mt-0.5">Play unassisted</div>
              </button>
            </div>
          </div>

          {/* Advisor simulations slider — only when hints are on */}
          <Show when={showHints()}>
            <div class="mb-8">
              <div class="flex items-center justify-between mb-3">
                <p class="text-gray-400 text-xs font-medium uppercase tracking-wider">
                  Advisor Quality
                </p>
                <span class="text-xs font-bold tabular-nums">
                  <span class={simQuality().color}>{simQuality().label}</span>
                  <span class="text-gray-500 ml-1.5">
                    {simulations().toLocaleString()} sims
                  </span>
                </span>
              </div>
              <input
                type="range"
                min="500"
                max="10000"
                step="500"
                value={simulations()}
                onInput={(e) => setSimulations(parseInt(e.currentTarget.value))}
                class="w-full h-2 rounded-full appearance-none cursor-pointer accent-amber-400
                       bg-gray-700"
              />
              <div class="flex justify-between text-gray-600 text-xs mt-1.5">
                <span>500</span>
                <span>10,000</span>
              </div>
            </div>
          </Show>

          {/* Start button */}
          <button
            onClick={() => {
              props.onStart(mode(), count(), simulations(), showHints());
            }}
            class="w-full py-5 bg-emerald-600 hover:bg-emerald-500 active:bg-emerald-700 text-white text-xl font-bold rounded-2xl transition-colors shadow-lg shadow-emerald-900/40"
          >
            Deal Cards
          </button>

          {/* Brief rules */}
          <div class="mt-6 text-center text-gray-600 text-xs space-y-1">
            <p>Collect 7 unique numbers without drawing a duplicate.</p>
            <p>Freeze opponents · Force 3 draws · Use your lifeline.</p>
          </div>
        </div>
      </div>
    </div>
  );
}
