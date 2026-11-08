<script lang="ts">
  import Scene from "../lib/Scene.svelte";
  import Stat, { type Tone } from "../lib/Stat.svelte";
  import Tach from "../lib/Tach.svelte";
  import { core } from "../lib/core.svelte";
  import { GAUGES, gauges, type Gauge } from "../lib/gauges.svelte";
  import { speedUnit, tempUnit, toSpeed, toTemp } from "../lib/settings.svelte";

  let { active }: { active: boolean } = $props();

  const showFps = new URLSearchParams(location.search).has("fps");
  let fps = $state(0);
  let editing = $state(false);

  type StatDef = { key: Gauge; value: string; unit: string; frac: number; tone: Tone };

  const range = (v: number, lo: number, hi: number) => (v - lo) / (hi - lo);
  const temp = (key: Gauge, c: number, lo: number, hi: number, warm: number, warn: number, danger: number): StatDef => ({
    key,
    value: toTemp(c).toFixed(0),
    unit: tempUnit(),
    frac: range(c, lo, hi),
    tone: c >= danger ? "danger" : c >= warn ? "warn" : c < warm ? "cold" : "ok",
  });

  const stats = $derived.by((): StatDef[] => {
    const t = core.telemetry;
    if (!t) return [];
    const v = t.battery_v;
    const f = t.fuel_pct;
    const all: StatDef[] = [
      temp("coolant", t.coolant_c, 40, 120, 70, 105, 112),
      temp("oil", t.oil_c, 40, 140, 70, 120, 130),
      temp("intake", t.intake_c, 0, 60, -99, 45, 55),
      {
        key: "battery",
        value: v.toFixed(1),
        unit: "V",
        frac: range(v, 11, 15),
        tone: v < 11.8 ? "danger" : v < 12.4 || v > 14.8 ? "warn" : "ok",
      },
      { key: "fuel", value: f.toFixed(0), unit: "%", frac: f / 100, tone: f < 8 ? "danger" : f < 15 ? "warn" : "ok" },
    ];
    return all.filter((s) => gauges[s.key]);
  });

  const launch = $derived(core.launch);
  const secs = (ms: number | null | undefined, digits: number) => (ms == null ? "–.––" : (ms / 1000).toFixed(digits));
</script>

<Scene {active} kmh={core.kmh} doors={core.doors} bind:fps />

<div class="dial glass">
  <Tach rpm={core.rpm} speed={toSpeed(core.kmh)} unit={speedUnit()} gear={core.telemetry?.gear ?? null} />
  {#if showFps}<span class="fps caps num">{fps} fps</span>{/if}
</div>

<div class="side">
  {#if gauges.launch}
    <div class="launch glass" class:running={launch?.running_ms != null}>
      <div class="row">
        <span class="caps">0–100 km/h</span>
        {#if launch?.running_ms != null}<span class="live caps">Timing</span>{/if}
      </div>
      <div class="time num">
        {launch?.running_ms != null ? secs(launch.running_ms, 1) : secs(launch?.last_ms, 2)}<small>s</small>
      </div>
      <div class="row best">
        <span class="caps">Best</span>
        <span class="num">{secs(launch?.best_ms, 2)} s</span>
      </div>
    </div>
  {/if}
  {#each stats as s (s.key)}
    <Stat label={GAUGES[s.key]} value={s.value} unit={s.unit} frac={s.frac} tone={s.tone} />
  {/each}
</div>

<div class="edit">
  {#if editing}
    <div class="chips glass">
      {#each Object.entries(GAUGES) as [key, label]}
        <button class:on={gauges[key as Gauge]} onclick={() => (gauges[key as Gauge] = !gauges[key as Gauge])}>
          {label}
        </button>
      {/each}
    </div>
  {/if}
  <button class="toggle glass" class:on={editing} onclick={() => (editing = !editing)} aria-label="Choose gauges">
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M4 14a8 8 0 0 1 16 0M12 14l4-4M3 18h18" />
    </svg>
  </button>
</div>

<style>
  .dial {
    position: absolute;
    top: var(--s-5);
    left: var(--s-5);
    width: min(380px, 60vh);
    padding: var(--s-3);
    border-radius: 50%;
  }
  .fps {
    position: absolute;
    top: 0;
    right: 0;
  }
  .side {
    position: absolute;
    top: calc(var(--s-4) + 56px);
    right: var(--s-4);
    bottom: calc(var(--mini) + var(--s-4));
    width: 248px;
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
    overflow: hidden;
  }
  .launch {
    padding: var(--s-3) var(--s-4);
    border-radius: var(--r-md);
    display: grid;
    gap: var(--s-1);
  }
  .launch.running {
    border-color: color-mix(in srgb, var(--accent) 70%, transparent);
    box-shadow:
      var(--e-2),
      0 0 24px -6px var(--accent);
  }
  .row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .live {
    display: inline-flex;
    align-items: center;
    gap: var(--s-1);
    color: var(--accent-hi);
  }
  .live::before {
    content: "";
    width: 8px;
    height: 8px;
    border-radius: var(--r-pill);
    background: currentColor;
    animation: pulse 600ms ease-in-out infinite alternate;
  }
  @keyframes pulse {
    to {
      opacity: 0.25;
    }
  }
  .time {
    font-size: var(--fs-xl);
    font-weight: 650;
    line-height: 1.1;
    letter-spacing: -0.02em;
  }
  .running .time {
    color: var(--accent-hi);
  }
  small {
    margin-left: 4px;
    font-size: var(--fs-md);
    font-weight: 500;
    color: var(--text-faint);
  }
  .best .num {
    font-size: var(--fs-sm);
    font-weight: 600;
    color: var(--text-dim);
  }
  .edit {
    position: absolute;
    left: var(--s-5);
    bottom: calc(var(--mini) + var(--s-4));
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: var(--s-2);
  }
  button {
    border: 1px solid var(--glass-border);
    color: var(--text-dim);
    font: inherit;
  }
  .toggle {
    width: var(--touch);
    height: var(--touch);
    display: grid;
    place-items: center;
    border-radius: var(--r-pill);
  }
  .toggle svg {
    width: 28px;
    height: 28px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.8;
    stroke-linecap: round;
  }
  .toggle.on {
    color: var(--accent-hi);
    border-color: var(--accent);
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
    max-width: 480px;
    padding: var(--s-3);
    border-radius: var(--r-lg);
  }
  .chips button {
    height: 48px;
    padding: 0 var(--s-4);
    border-radius: var(--r-pill);
    background: rgb(255 255 255 / 0.04);
    font-size: var(--fs-sm);
    font-weight: 500;
  }
  .chips button.on {
    background: var(--accent-soft);
    border-color: var(--accent);
    color: var(--text);
  }
  @media (prefers-reduced-motion: reduce) {
    .live::before {
      animation: none;
    }
  }
</style>
