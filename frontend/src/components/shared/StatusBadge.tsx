import type { PlayerStatus } from "@app/engine";

const CFG: Record<
  PlayerStatus["type"],
  { ring: string; label: string }
> = {
  Active:  { ring: "bg-emerald-500/15 text-emerald-400 border-emerald-500/30", label: "Active"   },
  Stopped: { ring: "bg-blue-500/15    text-blue-400    border-blue-500/30",    label: "Stopped"  },
  Busted:  { ring: "bg-red-500/15     text-red-400     border-red-500/30",     label: "Bust!"    },
  Flip7:   { ring: "bg-amber-500/15   text-amber-400   border-amber-500/30",   label: "FLIP 7!"  },
};

export function StatusBadge(props: { status: PlayerStatus }) {
  const c = () => CFG[props.status.type];
  return (
    <span
      class={`text-xs px-2 py-0.5 rounded-full border font-semibold ${c().ring}`}
    >
      {c().label}
    </span>
  );
}
