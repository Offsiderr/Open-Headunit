<script lang="ts">
  import { flip } from "svelte/animate";
  import { fly } from "svelte/transition";
  import type { Notification } from "../api";
  import { dismiss } from "./core.svelte";

  let { toasts }: { toasts: Notification[] } = $props();
</script>

{#each toasts as t (t.id)}
  <button
    class="toast glass"
    onclick={() => dismiss(t.id)}
    animate:flip={{ duration: 200 }}
    transition:fly={{ y: -24, duration: 250 }}
  >
    <span class="icon">
      <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 5h16v11H9l-5 4V5Z" /></svg>
    </span>
    <span class="text">
      <span class="caps">{t.app}</span>
      <b>{t.title}</b>
      <span class="body">{t.body}</span>
    </span>
  </button>
{/each}

<style>
  .toast {
    display: flex;
    align-items: center;
    gap: var(--s-4);
    padding: var(--s-3) var(--s-4);
    border-radius: var(--r-lg);
    font: inherit;
    color: inherit;
    text-align: left;
  }
  .icon {
    width: 48px;
    height: 48px;
    flex: none;
    display: grid;
    place-items: center;
    border-radius: var(--r-md);
    background: var(--accent-soft);
    color: var(--accent-hi);
  }
  svg {
    width: 24px;
    height: 24px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.8;
    stroke-linejoin: round;
  }
  .text {
    min-width: 0;
    display: grid;
  }
  .text > * {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  b {
    font-weight: 600;
  }
  .body {
    font-size: var(--fs-sm);
    color: var(--text-dim);
  }
</style>
