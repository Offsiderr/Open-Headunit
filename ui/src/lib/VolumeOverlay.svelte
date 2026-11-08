<script lang="ts">
  import { fly } from "svelte/transition";
  import { volume } from "./core.svelte";

  let shown = $state(false);

  $effect(() => {
    if (!volume.rev) return;
    shown = true;
    const t = setTimeout(() => (shown = false), 1500);
    return () => clearTimeout(t);
  });

  const level = $derived(volume.muted ? 0 : volume.level);
  const waves = $derived(
    volume.muted ? "M16 9.5l5 5M21 9.5l-5 5" : level === 0 ? "" : level < 50 ? "M16 9.5a3.5 3.5 0 0 1 0 5" : "M16 9.5a3.5 3.5 0 0 1 0 5M18.5 7a7 7 0 0 1 0 10",
  );
</script>

{#if shown}
  <div class="volume glass" class:muted={volume.muted} transition:fly={{ y: -24, duration: 250 }}>
    <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 9.5v5h3.5L12 18V6L7.5 9.5H4Z{waves}" /></svg>
    <div class="track"><div style:transform="scaleX({level / 100})"></div></div>
    <span class="num">{volume.muted ? "Mute" : level}</span>
  </div>
{/if}

<style>
  .volume {
    position: fixed;
    top: var(--s-5);
    left: 50%;
    translate: -50% 0;
    width: min(480px, calc(100% - 2 * var(--s-4)));
    box-sizing: border-box;
    display: flex;
    align-items: center;
    gap: var(--s-4);
    padding: var(--s-3) var(--s-5);
    border-radius: var(--r-pill);
    z-index: 10;
    pointer-events: none;
  }
  svg {
    width: 28px;
    height: 28px;
    flex: none;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.8;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .track {
    flex: 1;
    height: 6px;
    border-radius: var(--r-pill);
    background: rgb(255 255 255 / 0.1);
    overflow: hidden;
  }
  .track div {
    height: 100%;
    background: var(--accent-hi);
    box-shadow: 0 0 8px var(--accent);
    transform-origin: left;
    transition: transform var(--dur) var(--ease);
  }
  span {
    width: 3em;
    text-align: right;
    font-weight: 600;
  }
  .muted {
    color: var(--text-dim);
  }
</style>
