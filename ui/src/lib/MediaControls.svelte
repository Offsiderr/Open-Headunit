<script lang="ts">
  import { mediaCommand } from "./core.svelte";

  let { playing, large = false }: { playing: boolean; large?: boolean } = $props();
</script>

<div class="controls" class:large>
  <button onclick={() => mediaCommand("prev")} aria-label="Previous">
    <svg viewBox="0 0 24 24"><path d="M6 5v14M19 5 9 12l10 7V5Z" /></svg>
  </button>
  <button class="play" onclick={() => mediaCommand("toggle")} aria-label={playing ? "Pause" : "Play"}>
    <svg viewBox="0 0 24 24"><path d={playing ? "M8 5v14M16 5v14" : "M7 4.5v15L19 12 7 4.5Z"} /></svg>
  </button>
  <button onclick={() => mediaCommand("next")} aria-label="Next">
    <svg viewBox="0 0 24 24"><path d="M18 5v14M5 5l10 7-10 7V5Z" /></svg>
  </button>
</div>

<style>
  .controls {
    --size: var(--touch);
    --play: var(--touch);
    --icon: 28px;
    display: flex;
    align-items: center;
    gap: var(--s-1);
  }
  .large {
    --size: 80px;
    --play: 104px;
    --icon: 36px;
    gap: var(--s-6);
  }
  button {
    width: var(--size);
    height: var(--size);
    display: grid;
    place-items: center;
    padding: 0;
    border: none;
    border-radius: var(--r-pill);
    background: none;
    color: var(--text);
    transition:
      background var(--dur-fast) var(--ease),
      transform var(--dur-fast) var(--ease);
  }
  button:active {
    background: rgb(255 255 255 / 0.1);
    transform: scale(0.92);
  }
  .play {
    width: var(--play);
    height: var(--play);
    background: var(--art, var(--accent));
    color: #fff;
    box-shadow: 0 0 0 1px rgb(255 255 255 / 0.15) inset, 0 8px 32px -6px var(--art, var(--accent));
    transition:
      background var(--dur-slow) var(--ease),
      box-shadow var(--dur-slow) var(--ease),
      transform var(--dur-fast) var(--ease);
  }
  .play:active {
    background: var(--art, var(--accent));
    filter: brightness(1.15);
  }
  .controls:not(.large) .play {
    border-radius: var(--r-md);
  }
  svg {
    width: var(--icon);
    height: var(--icon);
    fill: none;
    stroke: currentColor;
    stroke-width: 2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .play svg {
    stroke-width: 2.6;
  }
  .large .play svg {
    width: 44px;
    height: 44px;
  }
</style>
