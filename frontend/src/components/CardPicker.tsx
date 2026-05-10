/**
 * CardPicker — used in Tracker mode to select the exact card that was flipped.
 *
 * Groups cards into three sections:
 *   Numbers (0–12)  ·  Action cards  ·  Modifier cards
 *
 * Each button shows the remaining count from the draw pile.
 * Cards with 0 remaining are faded but still clickable (tracking may lag).
 */
import { For } from "solid-js";
import type { Card, Deck } from "@app/engine";
import { NUM_BG } from "./shared/Cards";

interface Props {
  deck: Deck;
  /** Called when the user taps a card. */
  onPick: (card: Card) => void;
}

// All possible cards in a standard Flip7 deck, grouped for display.
const NUMBER_CARDS: Card[] = Array.from({ length: 13 }, (_, i) => ({
  type: "Number",
  value: i,
})) as Card[];

const ACTION_CARDS: { card: Card; label: string; emoji: string }[] = [
  {
    card: { type: "Action", value: { type: "Lifeline" } },
    label: "Lifeline",
    emoji: "🛡️",
  },
  {
    card: { type: "Action", value: { type: "Freeze" } },
    label: "Freeze",
    emoji: "🧊",
  },
  {
    card: { type: "Action", value: { type: "Tap3" } },
    label: "Tap 3",
    emoji: "👆",
  },
];

const MODIFIER_CARDS: { card: Card; label: string }[] = [
  { card: { type: "Modifier", value: { type: "Add", value: 2 } }, label: "+2" },
  { card: { type: "Modifier", value: { type: "Add", value: 4 } }, label: "+4" },
  { card: { type: "Modifier", value: { type: "Add", value: 6 } }, label: "+6" },
  { card: { type: "Modifier", value: { type: "Add", value: 8 } }, label: "+8" },
  {
    card: { type: "Modifier", value: { type: "Add", value: 10 } },
    label: "+10",
  },
  { card: { type: "Modifier", value: { type: "Multiply2" } }, label: "×2" },
];

function countInPile(draw: Card[], card: Card): number {
  return draw.filter((c) => {
    if (c.type !== card.type) return false;
    if (c.type === "Number" && card.type === "Number")
      return c.value === card.value;
    if (c.type === "Action" && card.type === "Action")
      return c.value.type === card.value.type;
    if (c.type === "Modifier" && card.type === "Modifier") {
      if (c.value.type !== card.value.type) return false;
      if (c.value.type === "Add" && card.value.type === "Add")
        return c.value.value === card.value.value;
      return true;
    }
    return false;
  }).length;
}

export default function CardPicker(props: Props) {
  const remaining = (card: Card) => countInPile(props.deck.draw_pile, card);

  return (
    <div class="space-y-4">
      {/* Numbers */}
      <div>
        <p class="text-xs text-gray-500 uppercase tracking-wider mb-2">
          Number Cards
        </p>
        <div class="grid grid-cols-5 gap-1.5">
          <For each={NUMBER_CARDS}>
            {(card) => {
              const left = () => remaining(card);
              const n = (card as { type: "Number"; value: number }).value;
              return (
                <button
                  onClick={() => props.onPick(card)}
                  class={`${NUM_BG[n] ?? "bg-gray-500"} ${
                    left() === 0
                      ? "opacity-30"
                      : "opacity-100 hover:brightness-110"
                  } relative aspect-2/3 rounded-lg flex flex-col items-center justify-center text-white font-bold shadow text-lg transition-all active:scale-95`}
                >
                  {n}
                  <span class="absolute bottom-1 right-1 text-[10px] opacity-70">
                    ×{left()}
                  </span>
                </button>
              );
            }}
          </For>
        </div>
      </div>

      {/* Actions */}
      <div>
        <p class="text-xs text-gray-500 uppercase tracking-wider mb-2">
          Action Cards
        </p>
        <div class="grid grid-cols-3 gap-2">
          <For each={ACTION_CARDS}>
            {({ card, label, emoji }) => {
              const left = () => remaining(card);
              return (
                <button
                  onClick={() => props.onPick(card)}
                  class={`${
                    left() === 0
                      ? "opacity-30"
                      : "opacity-100 hover:bg-gray-600"
                  } bg-gray-700 border border-gray-600 rounded-xl p-3 flex flex-col items-center gap-1 text-white transition-all active:scale-95`}
                >
                  <span class="text-xl">{emoji}</span>
                  <span class="text-xs font-bold">{label}</span>
                  <span class="text-[10px] text-gray-400">×{left()} left</span>
                </button>
              );
            }}
          </For>
        </div>
      </div>

      {/* Modifiers */}
      <div>
        <p class="text-xs text-gray-500 uppercase tracking-wider mb-2">
          Modifier Cards
        </p>
        <div class="grid grid-cols-3 gap-2">
          <For each={MODIFIER_CARDS}>
            {({ card, label }) => {
              const left = () => remaining(card);
              return (
                <button
                  onClick={() => props.onPick(card)}
                  class={`${
                    left() === 0
                      ? "opacity-30"
                      : "opacity-100 hover:bg-violet-700"
                  } bg-indigo-700 border border-indigo-600 rounded-xl py-2.5 flex flex-col items-center gap-0.5 text-white transition-all active:scale-95`}
                >
                  <span class="font-bold text-sm">{label}</span>
                  <span class="text-[10px] text-indigo-300">
                    ×{left()} left
                  </span>
                </button>
              );
            }}
          </For>
        </div>
      </div>
    </div>
  );
}
