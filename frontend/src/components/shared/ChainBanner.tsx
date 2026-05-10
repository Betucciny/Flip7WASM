interface Props {
  currentPlayer: number;
  turnHolder: number;
}

export function ChainBanner(props: Props) {
  return (
    <div class="bg-violet-900/30 border border-violet-500/40 rounded-xl px-3 py-2 text-xs text-violet-300">
      ⛓️ <strong>Chain:</strong> P{props.turnHolder}'s Tap3 → P
      {props.currentPlayer} resolves a queued effect
    </div>
  );
}
