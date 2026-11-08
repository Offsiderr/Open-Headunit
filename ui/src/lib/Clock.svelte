<script lang="ts">
  import { clock, formatTime } from "./settings.svelte";
</script>

<div class="clock glass num">{formatTime(clock.now)}</div>

<style>
  .clock {
    position: absolute;
    top: var(--s-4);
    right: var(--s-4);
    padding: var(--s-2) var(--s-4);
    border-radius: var(--r-pill);
    font-size: var(--fs-md);
    font-weight: 600;
    z-index: 2;
    pointer-events: none;
  }
</style>
