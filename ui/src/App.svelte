<script lang="ts">
  import Home from "./apps/Home.svelte";
  import Maps from "./apps/Maps.svelte";
  import Media from "./apps/Media.svelte";
  import Settings from "./apps/Settings.svelte";
  import CallCard from "./lib/CallCard.svelte";
  import Clock from "./lib/Clock.svelte";
  import MiniPlayer from "./lib/MiniPlayer.svelte";
  import Toasts from "./lib/Toasts.svelte";
  import VolumeOverlay from "./lib/VolumeOverlay.svelte";
  import { core, onInput, phone } from "./lib/core.svelte";
  import { brightness } from "./lib/settings.svelte";

  const apps = [
    { name: "Home", icon: "M3 11 12 4l9 7M5.5 9v11h13V9M10 20v-6h4v6" },
    { name: "Maps", icon: "M9 4 3 6v14l6-2 6 2 6-2V4l-6 2-6-2ZM9 4v14M15 6v14" },
    { name: "Media", icon: "M9 18V5l11-2v13M9 18a3 3 0 1 1-6 0 3 3 0 0 1 6 0ZM20 16a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z" },
    {
      name: "Settings",
      icon: "M4 6h9M17 6h3M17 6a2 2 0 1 1-4 0 2 2 0 0 1 4 0ZM4 12h3M11 12h9M11 12a2 2 0 1 1-4 0 2 2 0 0 1 4 0ZM4 18h10M18 18h2M18 18a2 2 0 1 1-4 0 2 2 0 0 1 4 0Z",
    },
  ];
  let focused = $state(0);
  let active = $state(0);
  let shown = $state(false);

  $effect(() => {
    if (!core.media) shown = false;
    else if (core.media.playing) shown = true;
  });
  const mini = $derived(shown && active <= 1);

  const move = (d: number) => (focused = (focused + apps.length + Math.sign(d)) % apps.length);

  $effect(() =>
    onInput((ev) => {
      if (phone.call && !phone.call.answered_at) return;
      if ("knob" in ev) move(ev.knob);
      else if (!("press" in ev)) return;
      else if (ev.press === "left") move(-1);
      else if (ev.press === "right") move(1);
      else if (ev.press === "select") active = focused;
      else if (ev.press === "back") active = focused = 0;
    }),
  );
</script>

<main style:--mini={mini ? "110px" : "0px"}>
  <section hidden={active !== 0}><Home active={active === 0} /></section>
  {#if active === 1}<section><Maps /></section>{/if}
  {#if active === 2}<section><Media /></section>{/if}
  {#if active === 3}<section><Settings /></section>{/if}
  {#if mini && core.media}<MiniPlayer media={core.media} />{/if}
  <Clock />
  <div class="top">
    {#if phone.call}<CallCard call={phone.call} />{/if}
    <Toasts toasts={phone.toasts} />
  </div>
</main>

<VolumeOverlay />

<div class="dim" style:opacity={1 - brightness() / 100}></div>

<nav class="glass" style:--count={apps.length} style:--active={active}>
  <span class="indicator"></span>
  {#each apps as app, i}
    <div class:focused={focused === i} class:active={active === i}>
      <svg viewBox="0 0 24 24" aria-hidden="true"><path d={app.icon} /></svg>
      <span>{app.name}</span>
    </div>
  {/each}
</nav>

<style>
  .dim {
    position: fixed;
    inset: 0;
    background: #000;
    pointer-events: none;
    z-index: 100;
    transition: opacity var(--dur-slow) var(--ease);
  }
  main {
    position: relative;
    flex: 1;
  }
  .top {
    position: absolute;
    top: var(--s-4);
    left: 0;
    right: 0;
    margin: 0 auto;
    width: min(560px, calc(100% - 2 * var(--s-4)));
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
    z-index: 2;
    pointer-events: none;
  }
  .top > :global(*) {
    pointer-events: auto;
  }
  section {
    position: absolute;
    inset: 0;
    animation: enter var(--dur-slow) var(--ease);
  }
  @keyframes enter {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
  }
  nav {
    position: relative;
    display: flex;
    height: var(--nav-h);
    padding: 0 var(--s-4);
    border-width: 1px 0 0;
    box-shadow: none;
  }
  .indicator {
    position: absolute;
    top: -1px;
    left: var(--s-4);
    width: calc((100% - 2 * var(--s-4)) / var(--count));
    height: 3px;
    transform: translateX(calc(100% * var(--active)));
    transition: transform var(--dur-slow) var(--ease);
  }
  .indicator::after {
    content: "";
    position: absolute;
    inset: 0 30%;
    border-radius: 0 0 var(--r-pill) var(--r-pill);
    background: var(--accent);
    box-shadow: 0 0 12px var(--accent);
  }
  nav div {
    position: relative;
    flex: 1;
    min-height: var(--touch);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--s-1);
    font-size: var(--fs-sm);
    font-weight: 500;
    color: var(--text-dim);
  }
  nav div::before {
    content: "";
    position: absolute;
    inset: var(--s-2);
    border-radius: var(--r-md);
    background: var(--accent-soft);
    box-shadow: inset 0 0 0 1px var(--accent);
    opacity: 0;
    transform: scale(0.94);
    transition:
      opacity var(--dur) var(--ease),
      transform var(--dur) var(--ease);
  }
  nav div > * {
    position: relative;
    opacity: 0.6;
    transition: opacity var(--dur) var(--ease);
  }
  nav svg {
    width: 28px;
    height: 28px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.8;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  nav .focused::before {
    opacity: 1;
    transform: none;
  }
  nav .focused > *,
  nav .active > * {
    opacity: 1;
  }
  nav .active {
    color: var(--text);
  }
  nav .active svg {
    color: var(--accent-hi);
  }
</style>
