<script lang="ts" module>
  export type Tone = "cold" | "ok" | "warn" | "danger";
</script>

<script lang="ts">
  let { label, value, unit, frac, tone }: { label: string; value: string; unit: string; frac: number; tone: Tone } =
    $props();
</script>

<div class="stat glass {tone}">
  <div class="row">
    <span class="caps">{label}</span>
    <span class="value num">{value}<small>{unit}</small></span>
  </div>
  <div class="bar"><div style:transform="scaleX({Math.min(Math.max(frac, 0), 1)})"></div></div>
</div>

<style>
  .stat {
    --tone: var(--ok);
    padding: var(--s-3) var(--s-4);
    border-radius: var(--r-md);
    display: grid;
    gap: var(--s-2);
  }
  .cold {
    --tone: #4fb3ff;
  }
  .warn {
    --tone: var(--warn);
  }
  .danger {
    --tone: var(--danger);
    border-color: color-mix(in srgb, var(--danger) 60%, transparent);
  }
  .row {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
  }
  .value {
    font-size: var(--fs-lg);
    font-weight: 600;
    line-height: 1;
  }
  .warn .value,
  .danger .value {
    color: var(--tone);
  }
  small {
    margin-left: 2px;
    font-size: var(--fs-sm);
    font-weight: 500;
    color: var(--text-faint);
  }
  .bar {
    height: 4px;
    border-radius: var(--r-pill);
    background: var(--n-700);
    overflow: hidden;
  }
  .bar div {
    height: 100%;
    border-radius: inherit;
    background: var(--tone);
    box-shadow: 0 0 8px var(--tone);
    transform-origin: left;
    transition:
      transform 300ms linear,
      background var(--dur) var(--ease);
  }
</style>
