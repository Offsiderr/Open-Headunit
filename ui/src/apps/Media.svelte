<script lang="ts">
  import { fade, fly } from "svelte/transition";
  import { art } from "../lib/art.svelte";
  import { core } from "../lib/core.svelte";
  import MediaControls from "../lib/MediaControls.svelte";

  const time = (ms: number) => {
    const s = Math.floor(ms / 1000);
    return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
  };
</script>

<div class="media" style:--art={art.color}>
  {#if core.media}
    {@const m = core.media}
    {@const progress = m.duration_ms ? m.position_ms / m.duration_ms : 0}
    {#key m.art}
      {#if m.art}<img class="backdrop" src={m.art} alt="" transition:fade={{ duration: 600 }} />{/if}
    {/key}
    <div class="shade"></div>

    <div class="stage">
      <div class="deck" class:playing={m.playing}>
        {#key m.art}
          <div class="slot" in:fly={{ x: -60, duration: 500, delay: 150 }} out:fade={{ duration: 200 }}>
            <div class="vinyl">
              <div class="grooves">
                {#if m.art}<img class="label" src={m.art} alt="" />{/if}
              </div>
            </div>
            {#if m.art}
              <img class="sleeve" src={m.art} alt="{m.title} cover" />
            {:else}
              <div class="sleeve blank"></div>
            {/if}
          </div>
        {/key}
      </div>

      <div class="info">
        <div class="status caps" class:playing={m.playing}>
          <span class="eq"><i></i><i></i><i></i></span>
          {m.playing ? "Now playing" : "Paused"}
        </div>
        {#key m.title}
          <div class="titles" in:fly={{ y: 12, duration: 400 }}>
            <h1>{m.title}</h1>
            <h2>{m.artist}</h2>
          </div>
        {/key}

        <div class="track">
          <div class="bar"><div class="fill" style:transform="scaleX({progress})"></div></div>
          <span class="head" style:left="{progress * 100}%"></span>
        </div>
        <div class="times num">
          <span>{time(m.position_ms)}</span>
          <span>-{time(Math.max(0, m.duration_ms - m.position_ms))}</span>
        </div>

        <MediaControls playing={m.playing} large />
      </div>
    </div>
  {:else}
    <div class="empty">
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path d="M9 18V5l11-2v13M9 18a3 3 0 1 1-6 0 3 3 0 0 1 6 0ZM20 16a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z" />
      </svg>
      <span class="caps">Nothing playing</span>
    </div>
  {/if}
</div>

<style>
  .media {
    --cover: min(52vh, 400px);
    position: relative;
    height: 100%;
    overflow: clip;
    background: var(--bg);
  }
  .backdrop {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    filter: blur(60px) saturate(1.5) brightness(0.55);
    transform: scale(1.3);
  }
  .shade {
    position: absolute;
    inset: 0;
    background:
      radial-gradient(ellipse at 25% 50%, transparent 0%, rgb(11 13 16 / 0.5) 70%),
      linear-gradient(to right, transparent 40%, rgb(11 13 16 / 0.55));
  }
  .stage {
    position: relative;
    height: 100%;
    display: grid;
    grid-template-columns: auto minmax(0, 560px);
    justify-content: center;
    align-items: center;
    gap: calc(var(--cover) * 0.55 + var(--s-7));
    padding: 0 var(--s-7);
  }

  .deck {
    position: relative;
    width: var(--cover);
    height: var(--cover);
  }
  .slot {
    position: absolute;
    inset: 0;
  }
  .vinyl {
    position: absolute;
    inset: 3%;
    border-radius: 50%;
    transform: translateX(18%);
    transition: transform 700ms var(--ease);
    box-shadow: 0 20px 50px -10px rgb(0 0 0 / 0.7);
  }
  .playing .vinyl {
    transform: translateX(52%);
  }
  .vinyl::after {
    content: "";
    position: absolute;
    inset: 0;
    border-radius: 50%;
    background: conic-gradient(
      from 30deg,
      transparent 0deg,
      rgb(255 255 255 / 0.1) 30deg,
      transparent 70deg,
      transparent 180deg,
      rgb(255 255 255 / 0.07) 220deg,
      transparent 260deg
    );
    pointer-events: none;
  }
  .grooves {
    position: absolute;
    inset: 0;
    border-radius: 50%;
    background:
      radial-gradient(circle, transparent 0 33%, rgb(0 0 0 / 0.6) 33.5% 35%, transparent 35.5%),
      repeating-radial-gradient(circle, #121315 0 1.5px, #1c1d20 2px 3px);
    animation: spin 4s linear infinite;
    animation-play-state: paused;
  }
  .playing .grooves {
    animation-play-state: running;
  }
  .label {
    position: absolute;
    inset: 33%;
    width: 34%;
    height: 34%;
    border-radius: 50%;
    object-fit: cover;
    -webkit-mask: radial-gradient(circle, transparent 0 7%, #000 8%);
    mask: radial-gradient(circle, transparent 0 7%, #000 8%);
  }
  @keyframes spin {
    to {
      transform: rotate(1turn);
    }
  }
  .sleeve {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    border-radius: var(--r-md);
    object-fit: cover;
    box-shadow:
      0 0 0 1px rgb(255 255 255 / 0.08),
      0 30px 60px -12px rgb(0 0 0 / 0.75),
      0 0 120px -30px var(--art);
    transition: box-shadow 600ms var(--ease);
  }
  .blank {
    background: linear-gradient(135deg, var(--n-700), var(--n-900));
  }

  .info {
    min-width: 0;
  }
  .status {
    display: inline-flex;
    align-items: center;
    gap: var(--s-3);
    color: var(--text-dim);
  }
  .eq {
    display: flex;
    align-items: flex-end;
    gap: 3px;
    height: 14px;
  }
  .eq i {
    width: 3px;
    height: 100%;
    border-radius: 2px;
    background: var(--art);
    transform-origin: bottom;
    transform: scaleY(0.3);
    animation: eq 900ms ease-in-out infinite alternate;
    animation-play-state: paused;
  }
  .eq i:nth-child(2) {
    animation-duration: 700ms;
    animation-delay: -300ms;
  }
  .eq i:nth-child(3) {
    animation-duration: 1100ms;
    animation-delay: -600ms;
  }
  .playing .eq i {
    animation-play-state: running;
  }
  @keyframes eq {
    from {
      transform: scaleY(0.25);
    }
    to {
      transform: scaleY(1);
    }
  }
  .titles {
    margin: var(--s-4) 0 var(--s-7);
  }
  h1 {
    margin: 0;
    font-size: clamp(36px, 6vh, 56px);
    font-weight: 700;
    line-height: 1.15;
    letter-spacing: -0.03em;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  h2 {
    margin: var(--s-2) 0 0;
    font-size: var(--fs-lg);
    font-weight: 400;
    color: var(--text-dim);
  }

  .track {
    position: relative;
  }
  .bar {
    height: 6px;
    border-radius: var(--r-pill);
    background: rgb(255 255 255 / 0.12);
    overflow: hidden;
  }
  .fill {
    height: 100%;
    background: var(--art);
    transform-origin: left;
    transition: transform 500ms linear;
  }
  .head {
    position: absolute;
    top: 50%;
    width: 14px;
    height: 14px;
    margin: -7px 0 0 -7px;
    border-radius: 50%;
    background: #fff;
    box-shadow: 0 0 12px 2px var(--art);
    transition: left 500ms linear;
  }
  .times {
    display: flex;
    justify-content: space-between;
    margin: var(--s-2) 0 var(--s-6);
    font-size: var(--fs-sm);
    color: var(--text-faint);
  }
  .info :global(.controls) {
    justify-content: center;
  }

  .empty {
    height: 100%;
    display: grid;
    place-content: center;
    justify-items: center;
    gap: var(--s-4);
  }
  .empty svg {
    width: 64px;
    height: 64px;
    fill: none;
    stroke: var(--text-faint);
    stroke-width: 1.5;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
</style>
