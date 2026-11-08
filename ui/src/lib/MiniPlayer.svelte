<script lang="ts">
  import { fly } from "svelte/transition";
  import type { MediaState } from "../api";
  import { art } from "./art.svelte";
  import MediaControls from "./MediaControls.svelte";

  let { media }: { media: MediaState } = $props();
</script>

<div class="mini glass" style:--art={art.color} transition:fly={{ y: 24, duration: 250 }}>
  <div class="tint"></div>
  {#if media.art}
    <img class="cover" class:playing={media.playing} src={media.art} alt="" />
  {/if}
  <div class="info">
    <b>{media.title}</b>
    <span>{media.artist}</span>
  </div>
  <MediaControls playing={media.playing} />
  <div class="bar">
    <div style:transform="scaleX({media.duration_ms ? media.position_ms / media.duration_ms : 0})"></div>
  </div>
</div>

<style>
  .mini {
    position: absolute;
    left: 50%;
    bottom: var(--s-4);
    translate: -50% 0;
    width: min(600px, calc(100% - 2 * var(--s-4)));
    box-sizing: border-box;
    display: flex;
    align-items: center;
    gap: var(--s-4);
    padding: var(--s-3);
    border-radius: var(--r-lg);
    overflow: hidden;
    z-index: 1;
  }
  .tint {
    position: absolute;
    inset: 0;
    background: radial-gradient(ellipse at 0% 50%, var(--art), transparent 60%);
    opacity: 0.25;
    pointer-events: none;
    transition: background var(--dur-slow) var(--ease);
  }
  .cover {
    position: relative;
    width: var(--touch);
    height: var(--touch);
    flex: none;
    border-radius: var(--r-sm);
    object-fit: cover;
    box-shadow: 0 4px 16px -4px rgb(0 0 0 / 0.6);
    filter: saturate(0.6) brightness(0.8);
    transition: filter var(--dur-slow) var(--ease);
  }
  .cover.playing {
    filter: none;
  }
  .info {
    position: relative;
    flex: 1;
    min-width: 0;
    display: grid;
  }
  .info > * {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  b {
    font-weight: 600;
  }
  .info span {
    font-size: var(--fs-sm);
    color: var(--text-dim);
  }
  .bar {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    height: 3px;
    background: rgb(255 255 255 / 0.08);
  }
  .bar div {
    height: 100%;
    background: var(--art);
    box-shadow: 0 0 8px var(--art);
    transform-origin: left;
    transition: transform 500ms linear;
  }
</style>
