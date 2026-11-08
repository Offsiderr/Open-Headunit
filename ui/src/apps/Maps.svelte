<script lang="ts">
  import { LngLatBounds, Map as MapLibre, Marker, setWorkerUrl, type GeoJSONSource } from "maplibre-gl";
  import "maplibre-gl/dist/maplibre-gl.css";
  import workerUrl from "maplibre-gl/dist/maplibre-gl-worker.mjs?url";
  import { untrack } from "svelte";
  import type { Place } from "../api";
  import { gps, nav, navCancel, navigate, navSave, navSearch } from "../lib/core.svelte";
  import { formatTime, settings, speedUnit, toSpeed } from "../lib/settings.svelte";

  const STYLE_URL = "https://tiles.openfreemap.org/styles/dark";
  const FOLLOW = { pitch: 55, zoom: 16 };
  const FIX_MS = 1000;
  const SEARCH_DEBOUNCE_MS = 300;

  const ICONS = {
    search: "M11 18a7 7 0 1 0 0-14 7 7 0 0 0 0 14ZM20 20l-4-4",
    close: "M6 6l12 12M18 6 6 18",
    pin: "M12 21s-6-5.5-6-11a6 6 0 0 1 12 0c0 5.5-6 11-6 11ZM12 12a2 2 0 1 0 0-4 2 2 0 0 0 0 4Z",
    recent: "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18ZM12 7v5l3 2",
    star: "M12 3l2.7 5.6 6.1.9-4.4 4.3 1 6.1L12 17l-5.4 2.9 1-6.1-4.4-4.3 6.1-.9Z",
    straight: "M12 20V4M6 10l6-6 6 6",
    left: "M17 20v-7a4 4 0 0 0-4-4H5M9 5 5 9l4 4",
    right: "M7 20v-7a4 4 0 0 1 4-4h8M15 5l4 4-4 4",
    "slight left": "M15 20v-7L8 6M8 12V6h6",
    "slight right": "M9 20v-7l7-7M10 6h6v6",
    uturn: "M16 20V9a4 4 0 0 0-8 0v7M4 12l4 4 4-4",
  };

  setWorkerUrl(workerUrl);

  type Pose = { lat: number; lon: number; heading: number };

  let container: HTMLDivElement;
  let input = $state<HTMLInputElement>();
  let map: MapLibre | undefined;
  let ready = $state(false);
  let follow = $state(true);
  let searching = $state(false);
  let text = $state(nav.query);
  let debounce: ReturnType<typeof setTimeout>;
  let from: Pose | null = null;
  let to: Pose | null = null;
  let t0 = 0;

  const route = $derived(nav.route);
  const step = $derived(route && nav.progress ? route.steps[nav.progress.step] : null);
  const last = $derived(!!route && !!nav.progress && nav.progress.step >= route.steps.length - 1);
  const thenStep = $derived(route && nav.progress && !last ? route.steps[nav.progress.step + 1] : null);
  const saved = (p: Place) => nav.destinations.saved.some((s) => s.lat === p.lat && s.lon === p.lon && s.name === p.name);

  function pose(now: number): Pose | null {
    if (!from || !to) return to;
    const k = Math.min(1, (now - t0) / FIX_MS);
    const dh = ((to.heading - from.heading + 540) % 360) - 180;
    return {
      lat: from.lat + (to.lat - from.lat) * k,
      lon: from.lon + (to.lon - from.lon) * k,
      heading: (from.heading + dh * k + 360) % 360,
    };
  }

  function icon(modifier: string, arrive: boolean) {
    if (arrive) return ICONS.pin;
    if (modifier === "uturn") return ICONS.uturn;
    if (modifier.startsWith("slight")) return ICONS[modifier as "slight left" | "slight right"];
    if (modifier.endsWith("left")) return ICONS.left;
    if (modifier.endsWith("right")) return ICONS.right;
    return ICONS.straight;
  }

  function distance(m: number) {
    if (settings.speedUnit === "mph") {
      const mi = m / 1609.344;
      return mi < 0.1 ? `${Math.round((m * 3.28084) / 50) * 50} ft` : `${mi.toFixed(mi < 10 ? 1 : 0)} mi`;
    }
    return m < 1000 ? `${Math.round(m / 10) * 10} m` : `${(m / 1000).toFixed(m < 10000 ? 1 : 0)} km`;
  }

  function duration(s: number) {
    const min = Math.max(1, Math.round(s / 60));
    return min < 60 ? `${min} min` : `${Math.floor(min / 60)} h ${min % 60} min`;
  }

  function away(p: Place) {
    const f = gps.current;
    if (!f?.fix) return "";
    const north = (p.lat - f.lat) * 111_320;
    const east = (p.lon - f.lon) * 111_320 * Math.cos((f.lat * Math.PI) / 180);
    return distance(Math.hypot(north, east));
  }

  function type(value: string) {
    text = value;
    clearTimeout(debounce);
    debounce = setTimeout(() => navSearch(text), SEARCH_DEBOUNCE_MS);
  }

  function open() {
    searching = true;
    setTimeout(() => input?.focus());
  }

  function close() {
    searching = false;
    input?.blur();
  }

  function go(p: Place) {
    close();
    follow = true;
    navigate(p);
  }

  $effect(() => {
    const f = gps.current;
    if (!f?.fix) return;
    const now = performance.now();
    const next = { lat: f.lat, lon: f.lon, heading: f.heading_deg };
    from = pose(now) ?? next;
    to = next;
    t0 = now;
  });

  $effect(() => {
    const start = untrack(() => pose(performance.now()));
    const m = new MapLibre({
      container,
      style: STYLE_URL,
      center: start ? [start.lon, start.lat] : [172.636, -43.531],
      bearing: start?.heading ?? 0,
      ...FOLLOW,
      attributionControl: false,
    });
    map = m;

    const el = document.createElement("div");
    el.innerHTML = `<svg viewBox="0 0 40 40" width="44" height="44" style="filter:drop-shadow(0 2px 4px #000a)">
      <path d="M20 3 L35 36 L20 28 L5 36 Z" fill="var(--accent)" stroke="#fff" stroke-width="2.5" stroke-linejoin="round"/></svg>`;
    const marker = new Marker({ element: el, rotationAlignment: "map", pitchAlignment: "map" });
    let added = false;

    m.on("load", () => {
      const labels = m.getStyle().layers.find((l) => l.type === "symbol")?.id;
      m.addSource("route", { type: "geojson", data: { type: "FeatureCollection", features: [] } });
      const line = { "line-join": "round", "line-cap": "round" } as const;
      m.addLayer(
        { id: "route-casing", type: "line", source: "route", layout: line, paint: { "line-color": "#0b2a5c", "line-width": 12 } },
        labels,
      );
      m.addLayer(
        { id: "route", type: "line", source: "route", layout: line, paint: { "line-color": "#4d8bf0", "line-width": 7 } },
        labels,
      );
      ready = true;
    });

    const unfollow = () => {
      follow = false;
      if (searching) close();
    };
    m.on("mousedown", unfollow);
    m.on("touchstart", unfollow);
    m.on("wheel", unfollow);

    let raf = requestAnimationFrame(function frame(now) {
      raf = requestAnimationFrame(frame);
      const p = pose(now);
      if (!p) return;
      marker.setLngLat([p.lon, p.lat]).setRotation(p.heading);
      if (!added) {
        marker.addTo(m);
        added = true;
      }
      if (follow && !m.isMoving()) m.jumpTo({ center: [p.lon, p.lat], bearing: p.heading });
    });

    return () => {
      cancelAnimationFrame(raf);
      m.remove();
      map = undefined;
      ready = false;
    };
  });

  $effect(() => {
    const m = map;
    if (!ready || !m) return;
    const coords = route?.geometry.map(([lat, lon]) => [lon, lat]) ?? [];
    (m.getSource("route") as GeoJSONSource).setData({
      type: "FeatureCollection",
      features: coords.length ? [{ type: "Feature", properties: {}, geometry: { type: "LineString", coordinates: coords } }] : [],
    });
    if (!route) return;
    const el = document.createElement("div");
    el.innerHTML = `<svg viewBox="0 0 24 24" width="40" height="40" style="filter:drop-shadow(0 2px 4px #000a)">
      <path d="${ICONS.pin}" fill="var(--danger)" stroke="#fff" stroke-width="1.5" stroke-linejoin="round"/></svg>`;
    const dest = new Marker({ element: el, anchor: "bottom" }).setLngLat([route.destination.lon, route.destination.lat]).addTo(m);
    return () => dest.remove();
  });

  function overview() {
    if (!route || !map) return;
    follow = false;
    const bounds = new LngLatBounds();
    for (const [lat, lon] of route.geometry) bounds.extend([lon, lat]);
    map.fitBounds(bounds, { padding: 120, pitch: 0, bearing: 0, duration: 800 });
  }

  function recenter() {
    follow = true;
    const p = pose(performance.now());
    if (p) map?.easeTo({ center: [p.lon, p.lat], bearing: p.heading, ...FOLLOW, duration: 600 });
  }
</script>

{#snippet row(p: Place, kind: "pin" | "recent" | "star")}
  <li>
    <button class="place" onclick={() => go(p)}>
      <svg viewBox="0 0 24 24" class="kind" class:fav={kind === "star"} aria-hidden="true"><path d={ICONS[kind]} /></svg>
      <span class="label">
        <b>{p.name}</b>
        {#if p.address}<small>{p.address}</small>{/if}
      </span>
      <small class="away num">{away(p)}</small>
    </button>
    <button class="save" class:on={saved(p)} onclick={() => navSave(p)} aria-label={saved(p) ? "Remove from saved" : "Save"}>
      <svg viewBox="0 0 24 24" aria-hidden="true"><path d={ICONS.star} /></svg>
    </button>
  </li>
{/snippet}

<div class="maps">
  <div class="map" bind:this={container}></div>

  {#if !gps.current?.fix}<div class="banner warn">No GPS fix</div>{/if}
  {#if nav.arrived}<div class="banner ok">Arrived at {nav.arrived.name}</div>{/if}

  {#if route && step}
    <div class="maneuver glass">
      <div class="now">
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d={icon(step.modifier, last)} /></svg>
        <div>
          <b class="num">{distance(nav.progress?.step_distance_m ?? 0)}</b>
          <span>{step.instruction}</span>
        </div>
      </div>
      {#if thenStep}
        <div class="then">
          Then
          <svg viewBox="0 0 24 24" aria-hidden="true"><path d={icon(thenStep.modifier, nav.progress!.step + 1 >= route.steps.length - 1)} /></svg>
          <span>{thenStep.instruction}</span>
        </div>
      {/if}
    </div>
  {:else}
    <div class="search glass" class:open={searching}>
      {#if searching}
        <div class="field">
          <svg viewBox="0 0 24 24" aria-hidden="true"><path d={ICONS.search} /></svg>
          <input
            bind:this={input}
            value={text}
            oninput={(e) => type(e.currentTarget.value)}
            onkeydown={(e) => {
              if (e.key === "Escape") close();
              else if (e.key === "Enter" && nav.results[0]) go(nav.results[0]);
            }}
            placeholder="Where to?"
            enterkeyhint="go"
            autocomplete="off"
            spellcheck="false"
          />
          <button class="icon" onclick={() => (text ? type("") : close())} aria-label={text ? "Clear" : "Close"}>
            <svg viewBox="0 0 24 24" aria-hidden="true"><path d={ICONS.close} /></svg>
          </button>
        </div>
        <div class="list">
          {#if nav.error}<p class="error">{nav.error}</p>{/if}
          {#if text.trim()}
            {#if nav.searching}
              <p class="hint">Searching…</p>
            {:else if !nav.results.length}
              <p class="hint">No places found</p>
            {/if}
            <ul>
              {#each nav.results as p (p)}{@render row(p, "pin")}{/each}
            </ul>
          {:else}
            {#if nav.destinations.saved.length}
              <h3 class="caps">Saved</h3>
              <ul>
                {#each nav.destinations.saved as p (p)}{@render row(p, "star")}{/each}
              </ul>
            {/if}
            {#if nav.destinations.recent.length}
              <h3 class="caps">Recent</h3>
              <ul>
                {#each nav.destinations.recent as p (p)}{@render row(p, "recent")}{/each}
              </ul>
            {/if}
            {#if !nav.destinations.saved.length && !nav.destinations.recent.length}
              <p class="hint">Search for an address or place to get directions</p>
            {/if}
          {/if}
        </div>
      {:else}
        <button class="pill" onclick={open}>
          <svg viewBox="0 0 24 24" aria-hidden="true"><path d={ICONS.search} /></svg>
          Where to?
        </button>
      {/if}
    </div>
  {/if}

  <div class="zoom">
    <button class="glass" onclick={() => map?.zoomIn()} aria-label="Zoom in">+</button>
    <button class="glass" onclick={() => map?.zoomOut()} aria-label="Zoom out">−</button>
  </div>

  {#if !follow}<button class="recenter" class:raised={!!route} onclick={recenter}>Recenter</button>{/if}

  {#if route && nav.progress}
    <div class="trip glass">
      <div class="eta">
        <b class="num">{formatTime(new Date(Date.now() + nav.progress.remaining_s * 1000))}</b>
        <span class="num">{duration(nav.progress.remaining_s)} · {distance(nav.progress.remaining_m)}</span>
      </div>
      <button class="overview" onclick={overview}>Overview</button>
      <button class="end" onclick={navCancel}>End</button>
    </div>
  {/if}

  <div class="speed glass num">
    <b>{Math.round(toSpeed(gps.current?.speed_kmh ?? 0))}</b><span>{speedUnit()}</span>
  </div>

  <div class="attrib">© OpenMapTiles © OpenStreetMap</div>
</div>

<style>
  .maps {
    position: relative;
    height: 100%;
    overflow: hidden;
  }
  .map {
    position: absolute;
    inset: 0;
    background: var(--bg);
  }
  svg {
    fill: none;
    stroke: currentColor;
    stroke-width: 2;
    stroke-linecap: round;
    stroke-linejoin: round;
    flex: none;
  }
  .banner {
    position: absolute;
    top: var(--s-4);
    left: 50%;
    transform: translateX(-50%);
    padding: var(--s-2) var(--s-5);
    border-radius: var(--r-pill);
    color: var(--bg);
    font-weight: 600;
    white-space: nowrap;
  }
  .banner.warn {
    background: var(--warn);
  }
  .banner.ok {
    top: calc(var(--s-4) + 48px);
    background: var(--ok);
  }
  button {
    font: inherit;
    color: var(--text);
    border-radius: var(--r-md);
  }

  .search {
    position: absolute;
    top: var(--s-4);
    left: var(--s-4);
    width: min(440px, calc(100% - 2 * var(--s-4) - var(--touch) - var(--s-4)));
    border-radius: var(--r-lg);
    overflow: hidden;
    display: flex;
    flex-direction: column;
    z-index: 1;
  }
  .search.open {
    bottom: calc(var(--s-4) + var(--mini, 0px));
  }
  .pill,
  .field {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    min-height: var(--touch);
    padding: 0 var(--s-3) 0 var(--s-5);
  }
  .pill {
    width: 100%;
    background: none;
    border: none;
    font-size: var(--fs-md);
    color: var(--text-dim);
    padding-right: var(--s-6);
  }
  .pill svg,
  .field > svg {
    width: 24px;
    height: 24px;
    color: var(--text-faint);
  }
  .field {
    border-bottom: 1px solid var(--glass-border);
  }
  input {
    flex: 1;
    min-width: 0;
    height: var(--touch);
    background: none;
    border: none;
    outline: none;
    font: inherit;
    font-size: var(--fs-md);
    color: var(--text);
  }
  input::placeholder {
    color: var(--text-faint);
  }
  .icon {
    width: 48px;
    height: 48px;
    display: grid;
    place-items: center;
    background: none;
    border: none;
    color: var(--text-dim);
  }
  .icon svg {
    width: 22px;
    height: 22px;
  }
  .list {
    flex: 1;
    overflow-y: auto;
    padding: var(--s-2) 0 var(--s-3);
  }
  h3 {
    margin: var(--s-3) var(--s-5) var(--s-1);
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  li {
    display: flex;
    align-items: center;
  }
  .place {
    flex: 1;
    min-width: 0;
    min-height: var(--touch);
    display: flex;
    align-items: center;
    gap: var(--s-4);
    padding: var(--s-2) var(--s-2) var(--s-2) var(--s-5);
    background: none;
    border: none;
    border-radius: 0;
    text-align: left;
  }
  .place:active,
  .icon:active,
  .save:active {
    background: var(--accent-soft);
  }
  .kind {
    width: 22px;
    height: 22px;
    color: var(--text-faint);
  }
  .kind.fav {
    color: var(--warn);
    fill: var(--warn);
  }
  .label {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .label b,
  .label small {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .label b {
    font-weight: 600;
  }
  small {
    font-size: var(--fs-xs);
    color: var(--text-faint);
  }
  .away {
    flex: none;
  }
  .save {
    width: var(--touch);
    height: var(--touch);
    display: grid;
    place-items: center;
    background: none;
    border: none;
    color: var(--text-faint);
  }
  .save svg {
    width: 22px;
    height: 22px;
  }
  .save.on {
    color: var(--warn);
  }
  .save.on svg {
    fill: var(--warn);
  }
  .hint,
  .error {
    margin: var(--s-3) var(--s-5);
    color: var(--text-faint);
  }
  .error {
    color: var(--danger);
  }

  .maneuver {
    position: absolute;
    top: var(--s-4);
    left: var(--s-4);
    width: min(440px, calc(100% - 2 * var(--s-4) - var(--touch) - var(--s-4)));
    border-radius: var(--r-lg);
    overflow: hidden;
  }
  .now {
    display: flex;
    align-items: center;
    gap: var(--s-4);
    padding: var(--s-4) var(--s-5);
    background: var(--accent);
  }
  .now svg {
    width: 56px;
    height: 56px;
    stroke-width: 2.5;
  }
  .now div {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .now b {
    font-size: var(--fs-xl);
    line-height: 1.1;
  }
  .now span {
    font-size: var(--fs-md);
    font-weight: 500;
  }
  .then {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-2) var(--s-5);
    font-size: var(--fs-sm);
    color: var(--text-dim);
  }
  .then svg {
    width: 20px;
    height: 20px;
  }
  .then span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .trip {
    position: absolute;
    bottom: calc(var(--s-4) + var(--mini, 0px));
    left: 50%;
    transform: translateX(-50%);
    transition: bottom var(--dur-slow) var(--ease);
    display: flex;
    align-items: center;
    gap: var(--s-3);
    padding: var(--s-2) var(--s-2) var(--s-2) var(--s-5);
    border-radius: var(--r-lg);
  }
  .eta {
    display: flex;
    flex-direction: column;
    margin-right: var(--s-3);
    white-space: nowrap;
  }
  .eta b {
    font-size: var(--fs-lg);
    color: var(--ok);
  }
  .eta span {
    color: var(--text-dim);
  }
  .overview,
  .end {
    min-height: var(--touch);
    padding: 0 var(--s-5);
    font-weight: 600;
    border: none;
  }
  .overview {
    background: var(--n-700);
  }
  .end {
    background: var(--danger);
  }

  .zoom {
    position: absolute;
    right: var(--s-4);
    top: 50%;
    transform: translateY(-50%);
    display: flex;
    flex-direction: column;
    gap: var(--s-3);
  }
  .zoom button {
    width: var(--touch);
    height: var(--touch);
    font-size: var(--fs-xl);
    line-height: 1;
  }
  .zoom button:active {
    background: var(--accent-soft);
  }
  .recenter {
    position: absolute;
    bottom: calc(var(--s-5) + var(--mini, 0px));
    left: 50%;
    transform: translateX(-50%);
    transition: bottom var(--dur-slow) var(--ease);
    min-height: var(--touch);
    padding: 0 var(--s-6);
    font-size: var(--fs-md);
    font-weight: 600;
    background: var(--accent);
    border: none;
    box-shadow: var(--e-2);
  }
  .recenter.raised {
    bottom: calc(var(--s-5) + var(--mini, 0px) + 96px);
  }
  .speed {
    position: absolute;
    left: var(--s-4);
    bottom: var(--s-4);
    padding: var(--s-3) var(--s-5);
    border-radius: var(--r-md);
    display: flex;
    align-items: baseline;
    gap: var(--s-2);
  }
  .speed b {
    font-size: var(--fs-xl);
    line-height: 1;
  }
  .speed span {
    color: var(--text-dim);
  }
  .attrib {
    position: absolute;
    right: var(--s-1);
    bottom: 2px;
    font-size: 10px;
    color: var(--text-faint);
  }
</style>
