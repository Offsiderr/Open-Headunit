<script lang="ts">
  import { fade, fly } from "svelte/transition";
  import type { Call } from "../api";
  import { onInput, phoneCommand } from "./core.svelte";

  let { call }: { call: Call } = $props();

  const HANGUP = "M3 15.5c5-5 13-5 18 0l-2.5 2.5-3.5-2v-2.5a10 10 0 0 0-6 0V16l-3.5 2L3 15.5Z";
  const ANSWER = "M5 3h4l2 5-2.5 1.5a11 11 0 0 0 6 6L16 13l5 2v4a2 2 0 0 1-2 2A16 16 0 0 1 3 5a2 2 0 0 1 2-2Z";

  let now = $state(Date.now());
  const elapsed = $derived(call.answered_at ? Math.max(0, Math.floor((now - call.answered_at) / 1000)) : 0);
  const clock = $derived(`${Math.floor(elapsed / 60)}:${String(elapsed % 60).padStart(2, "0")}`);

  $effect(() => {
    if (!call.answered_at) return;
    now = Date.now();
    const id = setInterval(() => (now = Date.now()), 1000);
    return () => clearInterval(id);
  });

  $effect(() =>
    onInput((ev) => {
      if (call.answered_at || !("press" in ev)) return;
      if (ev.press === "select") phoneCommand("accept");
      else if (ev.press === "back") phoneCommand("decline");
    }),
  );
</script>

{#if call.answered_at}
  <div class="active glass" transition:fly={{ y: -24, duration: 250 }}>
    <span class="dot"></span>
    <span class="who"><b>{call.name}</b><span class="num">{clock}</span></span>
    <button class="end small" onclick={() => phoneCommand("hangup")} aria-label="End call">
      <svg viewBox="0 0 24 24"><path d={HANGUP} /></svg>
    </button>
  </div>
{:else}
  <div class="backdrop" transition:fade={{ duration: 200 }}>
    <div class="ringing glass" transition:fly={{ y: 24, duration: 250 }}>
      <span class="caps">Incoming call</span>
      <span class="avatar">{call.name.charAt(0)}</span>
      <b class="name">{call.name}</b>
      <span class="number num">{call.number}</span>
      <div class="actions">
        <button class="end" onclick={() => phoneCommand("decline")} aria-label="Decline">
          <svg viewBox="0 0 24 24"><path d={HANGUP} /></svg>
        </button>
        <button class="accept" onclick={() => phoneCommand("accept")} aria-label="Accept">
          <svg viewBox="0 0 24 24"><path d={ANSWER} /></svg>
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    display: grid;
    place-items: center;
    background: rgb(0 0 0 / 0.5);
    z-index: 10;
  }
  .ringing {
    width: min(420px, calc(100% - 2 * var(--s-4)));
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: var(--s-6);
    border-radius: var(--r-lg);
  }
  .avatar {
    width: 96px;
    height: 96px;
    margin: var(--s-5) 0 var(--s-4);
    display: grid;
    place-items: center;
    border-radius: var(--r-pill);
    background: var(--accent-soft);
    color: var(--accent-hi);
    font-size: var(--fs-xl);
    font-weight: 600;
    animation: ring 1.6s var(--ease) infinite;
  }
  @keyframes ring {
    from {
      box-shadow: 0 0 0 0 var(--accent-soft);
    }
    to {
      box-shadow: 0 0 0 24px transparent;
    }
  }
  .name {
    font-size: var(--fs-lg);
    font-weight: 600;
  }
  .number {
    color: var(--text-dim);
  }
  .actions {
    display: flex;
    gap: var(--s-8);
    margin-top: var(--s-6);
  }
  button {
    width: 80px;
    height: 80px;
    display: grid;
    place-items: center;
    padding: 0;
    border: none;
    border-radius: var(--r-pill);
    color: #fff;
    transition: transform var(--dur-fast) var(--ease);
  }
  button:active {
    transform: scale(0.92);
  }
  .accept {
    background: var(--ok);
    box-shadow: 0 8px 32px -6px var(--ok);
  }
  .end {
    background: var(--danger);
    box-shadow: 0 8px 32px -6px var(--danger);
  }
  button.small {
    width: 48px;
    height: 48px;
    box-shadow: none;
  }
  svg {
    width: 32px;
    height: 32px;
    fill: currentColor;
  }
  .small svg {
    width: 24px;
    height: 24px;
  }
  .active {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    padding: var(--s-2) var(--s-2) var(--s-2) var(--s-4);
    border-radius: var(--r-pill);
  }
  .dot {
    width: 10px;
    height: 10px;
    flex: none;
    border-radius: var(--r-pill);
    background: var(--ok);
    box-shadow: 0 0 8px var(--ok);
  }
  .who {
    flex: 1;
    min-width: 0;
    display: flex;
    gap: var(--s-3);
    align-items: baseline;
  }
  .who b {
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .who span {
    color: var(--ok);
  }
</style>
