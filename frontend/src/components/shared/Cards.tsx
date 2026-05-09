/**
 * Colored number and modifier card primitives shared across both game boards.
 *
 * All Tailwind bg-* class strings are written out explicitly so the JIT
 * scanner picks them up. Do NOT generate them with template literals.
 */

// ── Number-card colour map (0–12) ─────────────────────────────────────────────

export const NUM_BG: Record<number, string> = {
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

// ── Components ────────────────────────────────────────────────────────────────

/** Small number card — used in the player-list rows. */
export function SmallNumberCard(props: { n: number }) {
  return (
    <div
      class={`${NUM_BG[props.n] ?? "bg-gray-500"} w-6 h-8 rounded flex items-center justify-center text-white text-[11px] font-bold shadow shrink-0`}
    >
      {props.n}
    </div>
  );
}

/** Large number card — used in the current-player detail panel. */
export function BigNumberCard(props: { n: number }) {
  return (
    <div
      class={`${NUM_BG[props.n] ?? "bg-gray-500"} w-14 h-20 rounded-xl flex items-center justify-center text-white text-2xl font-black shadow-lg shrink-0`}
    >
      {props.n}
    </div>
  );
}

/** Modifier pill — purple for Add, violet for Multiply2. */
export function ModifierTag(props: {
  mod: { type: "Add"; value: number } | { type: "Multiply2" };
}) {
  const label = props.mod.type === "Add" ? `+${props.mod.value}` : "×2";
  const cls =
    props.mod.type === "Multiply2"
      ? "bg-violet-600 border-violet-500"
      : "bg-indigo-600 border-indigo-500";
  return (
    <div
      class={`${cls} px-2.5 py-1 rounded-md text-white text-xs font-bold border`}
    >
      {label}
    </div>
  );
}
