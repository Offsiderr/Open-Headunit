<script lang="ts">
  let { rpm, speed, unit, gear }: { rpm: number; speed: number; unit: string; gear: number | null } = $props();

  const max = 8000;
  const shift = 6800;
  const redline = 7500;
  const SWEEP = 270;
  const R = 84;
  const LIGHTS = 9;
  const LIGHTS_FROM = 5000;

  const angle = (v: number) => -SWEEP / 2 + (SWEEP * Math.min(Math.max(v, 0), max)) / max;
  const point = (deg: number, r: number) => {
    const a = (deg * Math.PI) / 180;
    return [100 + r * Math.sin(a), 100 - r * Math.cos(a)];
  };
  const arc = (from: number, to: number, r = R) => {
    const [x0, y0] = point(angle(from), r);
    const [x1, y1] = point(angle(to), r);
    return `M${x0} ${y0}A${r} ${r} 0 ${angle(to) - angle(from) > 180 ? 1 : 0} 1 ${x1} ${y1}`;
  };

  const ticks = Array.from({ length: max / 250 + 1 }, (_, i) => {
    const v = i * 250;
    const major = v % 1000 === 0;
    const a = angle(v);
    const [x0, y0] = point(a, major ? 70 : 74);
    const [x1, y1] = point(a, 77);
    return { x0, y0, x1, y1, major, red: v >= redline };
  });
  const labels = Array.from({ length: max / 1000 + 1 }, (_, i) => {
    const [x, y] = point(angle(i * 1000), 60);
    return { x, y, text: i, red: i * 1000 >= redline };
  });
  const lights = Array.from({ length: LIGHTS }, (_, i) => {
    const at = LIGHTS_FROM + ((shift - LIGHTS_FROM) * i) / (LIGHTS - 1);
    return { at, tone: i < 3 ? "ok" : i < 6 ? "warn" : "danger" };
  });

  const track = arc(0, max);
  const frac = $derived(Math.min(Math.max(rpm, 0), max) / max);
  const tone = $derived(rpm >= redline ? "danger" : rpm >= shift ? "warn" : "accent");
</script>

<div class="tach" class:flash={rpm >= shift}>
  <svg viewBox="0 0 200 200" aria-hidden="true">
    <path class="track" d={track} />
    <path class="redzone" d={arc(redline, max)} />
    <path class="glow {tone}" d={track} pathLength="1" stroke-dasharray="{frac} 1" />
    <path class="value {tone}" d={track} pathLength="1" stroke-dasharray="{frac} 1" />
    {#each ticks as t}
      <line class:major={t.major} class:red={t.red} x1={t.x0} y1={t.y0} x2={t.x1} y2={t.y1} />
    {/each}
    {#each labels as l}
      <text class:red={l.red} x={l.x} y={l.y}>{l.text}</text>
    {/each}
  </svg>
  <div class="lights">
    {#each lights as l}
      <span class="{l.tone}" class:on={rpm >= l.at}></span>
    {/each}
  </div>
  <div class="center">
    <span class="speed num">{speed.toFixed(0)}</span>
    <span class="caps">{unit}</span>
  </div>
  <div class="foot">
    <span class="gear num" class:neutral={!gear}>{gear ? gear : "N"}</span>
    <span class="rpm num">{(rpm / 1000).toFixed(1)}<small>×1000 rpm</small></span>
  </div>
</div>

<style>
  .tach {
    --tone-accent: var(--accent-hi);
    --tone-warn: var(--warn);
    --tone-danger: var(--danger);
    position: relative;
    width: 100%;
    aspect-ratio: 1;
    container-type: inline-size;
  }
  svg {
    position: absolute;
    inset: 0;
    overflow: visible;
  }
  path {
    fill: none;
    stroke-linecap: round;
  }
  .track {
    stroke: rgb(255 255 255 / 0.06);
    stroke-width: 8;
  }
  .redzone {
    stroke: color-mix(in srgb, var(--danger) 35%, transparent);
    stroke-width: 8;
    stroke-linecap: butt;
  }
  .value {
    stroke-width: 8;
    transition: stroke-dasharray 60ms linear, stroke var(--dur) var(--ease);
  }
  .glow {
    stroke-width: 18;
    opacity: 0.18;
    transition: stroke-dasharray 60ms linear, stroke var(--dur) var(--ease);
  }
  .accent {
    stroke: var(--tone-accent);
  }
  .warn {
    stroke: var(--tone-warn);
  }
  .danger {
    stroke: var(--tone-danger);
  }
  line {
    stroke: var(--n-500);
    stroke-width: 0.8;
  }
  line.major {
    stroke: var(--n-200);
    stroke-width: 1.6;
  }
  line.red {
    stroke: var(--danger);
  }
  text {
    fill: var(--text-dim);
    font-size: 10px;
    font-weight: 600;
    text-anchor: middle;
    dominant-baseline: central;
  }
  text.red {
    fill: var(--danger);
  }
  .lights {
    position: absolute;
    top: 31%;
    left: 50%;
    translate: -50% 0;
    display: flex;
    gap: 2.2cqi;
  }
  .lights span {
    width: 2.6cqi;
    height: 2.6cqi;
    border-radius: var(--r-pill);
    background: var(--n-700);
    transition: background 60ms linear, box-shadow 60ms linear;
  }
  .lights .on.ok {
    background: var(--ok);
    box-shadow: 0 0 8px var(--ok);
  }
  .lights .on.warn {
    background: var(--warn);
    box-shadow: 0 0 8px var(--warn);
  }
  .lights .on.danger {
    background: var(--danger);
    box-shadow: 0 0 8px var(--danger);
  }
  .flash .lights span {
    animation: flash 120ms steps(2, jump-none) infinite alternate;
  }
  @keyframes flash {
    to {
      background: var(--n-700);
      box-shadow: none;
    }
  }
  .center {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding-top: 4cqi;
  }
  .speed {
    font-size: 25cqi;
    font-weight: 650;
    line-height: 1;
    letter-spacing: -0.04em;
  }
  .foot {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 6%;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1cqi;
  }
  .gear {
    min-width: 11cqi;
    padding: 0.5cqi 2.5cqi;
    box-sizing: border-box;
    border-radius: var(--r-sm);
    background: var(--accent-soft);
    box-shadow: inset 0 0 0 1px var(--accent);
    color: var(--text);
    font-size: 7.5cqi;
    font-weight: 700;
    text-align: center;
  }
  .gear.neutral {
    background: var(--ok-soft);
    box-shadow: inset 0 0 0 1px var(--ok);
    color: var(--ok);
  }
  .flash .gear {
    background: var(--warn-soft);
    box-shadow: inset 0 0 0 1px var(--warn);
    color: var(--warn);
  }
  .rpm {
    font-size: 4.2cqi;
    font-weight: 600;
    color: var(--text-dim);
  }
  small {
    margin-left: 1cqi;
    font-size: 3cqi;
    letter-spacing: var(--tracking-caps);
    text-transform: uppercase;
    color: var(--text-faint);
  }
  @media (prefers-reduced-motion: reduce) {
    .flash .lights span {
      animation: none;
    }
  }
</style>
