import { createSignal, onMount, Show } from "solid-js";
import {
  initEngine,
  newGame,
  applyAction,
  getRecommendation,
  endRound,
  type Action,
  type GameState,
  type Recommendation,
} from "@app/engine";
import { findWinner } from "./utils/scoring";
import StartScreen, { type GameMode } from "./components/StartScreen";
import SimulatorBoard from "./components/SimulatorBoard";
import TrackerBoard from "./components/TrackerBoard";
import GameOverScreen from "./components/GameOverScreen";

// ── Screen state machine ──────────────────────────────────────────────────────

type Screen = "loading" | "start" | "game" | "gameover";

const MAX_HISTORY = 30;

export default function App() {
  const [screen, setScreen] = createSignal<Screen>("loading");
  const [mode, setMode] = createSignal<GameMode>("simulator");
  const [gameState, setGameState] = createSignal<GameState | null>(null);
  const [rec, setRec] = createSignal<Recommendation | null>(null);
  /** Undo history — capped at MAX_HISTORY entries. Each entry is the state
   *  before the corresponding action so pressing Undo restores it exactly. */
  const [history, setHistory] = createSignal<GameState[]>([]);
  const [numberRounds, setNumberRounds] = createSignal<number>(500);

  // ── WASM initialisation ───────────────────────────────────────────────────

  onMount(async () => {
    await initEngine();
    setScreen("start");
  });

  // ── Helpers ───────────────────────────────────────────────────────────────

  function refreshRec(s: GameState) {
    const recommendation =
      s.phase.type === "Playing" ? getRecommendation(s, numberRounds()) : null;
    console.log("Recommendation:", recommendation);
    setRec(recommendation);
  }

  function pushHistory(s: GameState) {
    setHistory((h) => [...h.slice(-(MAX_HISTORY - 1)), s]);
  }

  // ── Handlers ─────────────────────────────────────────────────────────────

  function handleStart(m: GameMode, playerCount: number, simulations: number) {
    setHistory([]);
    setMode(m);
    setNumberRounds(simulations);
    const s = newGame(playerCount);
    setGameState(s);
    refreshRec(s);
    setScreen("game");
  }

  function handleAction(action: Action) {
    const s = gameState();
    if (!s) return;
    try {
      const next = applyAction(s, s.current_player, action);
      console.log("Total cards in deck:", next.deck.draw_pile.length);
      console.log(
        "Total cards in discard pile:",
        next.deck.discard_pile.length,
      );
      console.log("State:", next);
      pushHistory(s); // save pre-action state for undo
      setGameState(next);
      refreshRec(next);
    } catch (e) {
      console.error("Action rejected by engine:", e);
    }
  }

  function handleUndo() {
    const hist = history();
    if (hist.length === 0) return;
    const prev = hist[hist.length - 1];
    setHistory((h) => h.slice(0, -1));
    setGameState(prev);
    refreshRec(prev);
  }

  function handleNextRound() {
    const s = gameState();
    if (!s) return;
    setHistory([]); // clear undo history on round transition
    const next = endRound(s);
    if (findWinner(next.players) >= 0) {
      setGameState(next);
      setScreen("gameover");
      return;
    }
    setGameState(next);
    refreshRec(next);
  }

  function handleEndGame() {
    const s = gameState();
    if (!s) return;
    setHistory([]);
    setGameState(endRound(s));
    setScreen("gameover");
  }

  function handleRestart() {
    setHistory([]);
    setGameState(null);
    setRec(null);
    setScreen("start");
  }

  // ── Render ────────────────────────────────────────────────────────────────

  return (
    <div class="min-h-screen bg-gray-950 text-gray-100">
      {/* Loading */}
      <Show when={screen() === "loading"}>
        <div class="min-h-screen flex items-center justify-center">
          <div class="text-center">
            <div class="text-6xl font-black mb-4">
              <span class="text-white">FLIP</span>
              <span class="text-amber-400"> 7</span>
            </div>
            <p class="text-gray-400 text-sm animate-pulse">Loading engine…</p>
          </div>
        </div>
      </Show>

      {/* Start screen */}
      <Show when={screen() === "start"}>
        <StartScreen onStart={handleStart} />
      </Show>

      {/* Game — Simulator */}
      <Show
        when={
          screen() === "game" && mode() === "simulator" && gameState() !== null
        }
      >
        <SimulatorBoard
          state={gameState()!}
          recommendation={rec()}
          onAction={handleAction}
          onNextRound={handleNextRound}
          onEndGame={handleEndGame}
          canUndo={history().length > 0}
          onUndo={handleUndo}
        />
      </Show>

      {/* Game — Tracker */}
      <Show
        when={
          screen() === "game" && mode() === "tracker" && gameState() !== null
        }
      >
        <TrackerBoard
          state={gameState()!}
          recommendation={rec()}
          onAction={handleAction}
          onNextRound={handleNextRound}
          onEndGame={handleEndGame}
          canUndo={history().length > 0}
          onUndo={handleUndo}
        />
      </Show>

      {/* Game over */}
      <Show when={screen() === "gameover" && gameState() !== null}>
        <GameOverScreen state={gameState()!} onRestart={handleRestart} />
      </Show>
    </div>
  );
}
