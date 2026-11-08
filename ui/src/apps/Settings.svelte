<script lang="ts">
  import { core, phone, phoneCommand, phoneSimulate, setVolume, volume } from "../lib/core.svelte";
  import {
    ACCENTS,
    EQ_BANDS,
    EQ_PRESETS,
    clock,
    formatTime,
    night,
    resetSettings,
    settings,
    speedUnit,
    tempUnit,
    toSpeed,
    toTemp,
  } from "../lib/settings.svelte";
  import Diagnostics from "./Diagnostics.svelte";

  const tabs = ["Audio", "Display", "Bluetooth", "Units & Clock", "Diagnostics", "System"];
  let tab = $state("Audio");

  function preset(name: string) {
    settings.eqPreset = name;
    settings.eq = [...EQ_PRESETS[name]];
  }

  function band(i: number, v: number) {
    settings.eq[i] = v;
    settings.eqPreset = Object.keys(EQ_PRESETS).find((k) => EQ_PRESETS[k].every((x, j) => x === settings.eq[j])) ?? "Custom";
  }

  const side = (v: number, l: string, r: string) => (v === 0 ? "Centre" : `${Math.abs(v)} ${v < 0 ? l : r}`);
</script>

{#snippet toggle(on: boolean, flip: () => void, disabled = false)}
  <button class="toggle" class:on role="switch" aria-checked={on} aria-label={on ? "On" : "Off"} {disabled} onclick={flip}><span></span></button>
{/snippet}

{#snippet seg(value: string, options: [string, string][], pick: (v: string) => void, disabled = false)}
  <span class="seg">
    {#each options as [v, label]}
      <button class:sel={value === v} {disabled} onclick={() => pick(v)}>{label}</button>
    {/each}
  </span>
{/snippet}

{#snippet slider(value: number, min: number, max: number, set: (v: number) => void, label: string)}
  <span class="row">
    <input type="range" {min} {max} {value} oninput={(e) => set(+e.currentTarget.value)} />
    <span class="num val">{label}</span>
  </span>
{/snippet}

<div class="settings">
  <aside>
    <h1>Settings</h1>
    {#each tabs as t}
      <button class="tab" class:sel={tab === t} onclick={() => (tab = t)}>{t}</button>
    {/each}
  </aside>

  <div class="panel">
    <h2>{tab}</h2>

    {#if tab === "Audio"}
      <div class="caps">Volume</div>
      <ul class="glass">
        <li>
          <span>Volume</span>
          <span class="row">
            <button onclick={() => setVolume(volume.level - 5)} aria-label="Volume down">−</button>
            <input
              type="range"
              min="0"
              max="100"
              value={volume.level}
              oninput={(e) => setVolume(+e.currentTarget.value, volume.muted)}
            />
            <button onclick={() => setVolume(volume.level + 5)} aria-label="Volume up">+</button>
            <span class="num val">{volume.muted ? "Mute" : volume.level}</span>
          </span>
        </li>
        <li>
          <span>Mute</span>
          {@render toggle(volume.muted, () => setVolume(volume.level, !volume.muted))}
        </li>
        <li>
          <span>Max volume at start-up<small>Caps the level restored when the unit powers on</small></span>
          {@render slider(settings.startupVolume, 0, 100, (v) => (settings.startupVolume = v), `${settings.startupVolume}`)}
        </li>
        <li>
          <span>Speed-compensated volume<small>Raises volume as road noise increases</small></span>
          {@render seg(
            settings.speedVolume,
            [["off", "Off"], ["low", "Low"], ["med", "Med"], ["high", "High"]],
            (v) => (settings.speedVolume = v as typeof settings.speedVolume),
          )}
        </li>
      </ul>

      <div class="caps">Equaliser</div>
      <ul class="glass">
        <li>
          <span>Preset</span>
          {@render seg(settings.eqPreset, Object.keys(EQ_PRESETS).map((k) => [k, k]), preset)}
        </li>
        <li class="eq">
          {#each EQ_BANDS as b, i}
            <label>
              <span class="num">{settings.eq[i] > 0 ? "+" : ""}{settings.eq[i]}</span>
              <input type="range" min="-10" max="10" value={settings.eq[i]} oninput={(e) => band(i, +e.currentTarget.value)} />
              <span class="caps">{b}</span>
            </label>
          {/each}
        </li>
        <li>
          <span>Loudness<small>Boosts lows and highs at low volume</small></span>
          {@render toggle(settings.loudness, () => (settings.loudness = !settings.loudness))}
        </li>
      </ul>

      <div class="caps">Sound field</div>
      <ul class="glass">
        <li>
          <span>Balance</span>
          {@render slider(settings.balance, -10, 10, (v) => (settings.balance = v), side(settings.balance, "L", "R"))}
        </li>
        <li>
          <span>Fade</span>
          {@render slider(settings.fade, -10, 10, (v) => (settings.fade = v), side(settings.fade, "Rear", "Front"))}
        </li>
      </ul>
    {:else if tab === "Display"}
      <div class="caps">Brightness</div>
      <ul class="glass">
        <li>
          <span>Brightness</span>
          {@render slider(settings.brightness, 10, 100, (v) => (settings.brightness = v), `${settings.brightness}%`)}
        </li>
        <li>
          <span>Night mode<small>Auto dims between 19:00 and 06:30{night() ? " · active now" : ""}</small></span>
          {@render seg(
            settings.nightMode,
            [["off", "Off"], ["auto", "Auto"], ["on", "On"]],
            (v) => (settings.nightMode = v as typeof settings.nightMode),
          )}
        </li>
        <li>
          <span>Night brightness</span>
          {@render slider(settings.nightBrightness, 10, 100, (v) => (settings.nightBrightness = v), `${settings.nightBrightness}%`)}
        </li>
      </ul>

      <div class="caps">Appearance</div>
      <ul class="glass">
        <li>
          <span>Accent colour</span>
          <span class="row">
            {#each Object.entries(ACCENTS) as [name, [c]]}
              <button
                class="swatch"
                class:sel={settings.accent === name}
                style:--c={c}
                aria-label={name}
                onclick={() => (settings.accent = name)}
              ></button>
            {/each}
          </span>
        </li>
        <li>
          <span>Reduce transparency<small>Solid panels instead of blur. Faster on the Pi</small></span>
          {@render toggle(settings.solid, () => (settings.solid = !settings.solid))}
        </li>
      </ul>
    {:else if tab === "Bluetooth"}
      <ul class="glass">
        <li>
          <span>Bluetooth</span>
          {@render toggle(settings.bluetooth, () => (settings.bluetooth = !settings.bluetooth))}
        </li>
        <li>
          <span>Device name</span>
          <input class="text" bind:value={settings.btName} disabled={!settings.bluetooth} maxlength="32" />
        </li>
        <li>
          <span>Discoverable<small>Lets new phones find and pair with the car</small></span>
          {@render toggle(settings.discoverable, () => (settings.discoverable = !settings.discoverable), !settings.bluetooth)}
        </li>
        <li>
          <span>Auto-connect<small>Reconnects to the last phone on start-up</small></span>
          {@render toggle(settings.autoConnect, () => (settings.autoConnect = !settings.autoConnect), !settings.bluetooth)}
        </li>
      </ul>

      <div class="caps">Paired devices</div>
      <ul class="glass">
        {#each settings.devices as d, i (d.addr)}
          <li>
            <span>
              {d.name}
              <small class="num">{d.addr}{settings.bluetooth && d.connected ? " · Connected" : ""}</small>
            </span>
            <span class="row">
              <button
                class="wide"
                disabled={!settings.bluetooth}
                onclick={() => settings.devices.forEach((x, j) => (x.connected = j === i && !d.connected))}
              >
                {d.connected ? "Disconnect" : "Connect"}
              </button>
              <button class="wide danger" onclick={() => settings.devices.splice(i, 1)}>Forget</button>
            </span>
          </li>
        {:else}
          <li><span class="dim">No paired devices. Turn on discoverable to pair a phone.</span></li>
        {/each}
      </ul>
    {:else if tab === "Units & Clock"}
      <div class="caps">Units</div>
      <ul class="glass">
        <li>
          <span>Speed<small>{toSpeed(100).toFixed(0)} {speedUnit()} = 100 km/h</small></span>
          {@render seg(settings.speedUnit, [["kmh", "km/h"], ["mph", "mph"]], (v) => (settings.speedUnit = v as typeof settings.speedUnit))}
        </li>
        <li>
          <span>Temperature<small>Coolant at 90 °C reads {toTemp(90).toFixed(0)} {tempUnit()}</small></span>
          {@render seg(settings.tempUnit, [["c", "°C"], ["f", "°F"]], (v) => (settings.tempUnit = v as typeof settings.tempUnit))}
        </li>
      </ul>

      <div class="caps">Clock</div>
      <ul class="glass">
        <li>
          <span>Format<small class="num">Now {formatTime(clock.now)}</small></span>
          {@render seg(settings.clock24 ? "24" : "12", [["24", "24-hour"], ["12", "12-hour"]], (v) => (settings.clock24 = v === "24"))}
        </li>
        <li>
          <span>Show seconds</span>
          {@render toggle(settings.clockSeconds, () => (settings.clockSeconds = !settings.clockSeconds))}
        </li>
      </ul>
    {:else if tab === "Diagnostics"}
      <Diagnostics />
    {:else}
      <div class="caps">Status</div>
      <ul class="glass">
        <li>
          <span>Core</span>
          <span class="pill" class:ok={core.connected}>
            <span class="dot"></span>{core.connected ? "Connected" : "Disconnected"}
          </span>
        </li>
        <li>
          <span>Reset settings<small>Restores every option on this page to its default</small></span>
          <button class="wide danger" onclick={resetSettings}>Reset</button>
        </li>
      </ul>

      <div class="caps">Debug</div>
      <ul class="glass">
        <li>
          <span>Notification</span>
          <button class="wide" onclick={() => phoneSimulate("notification")}>Send</button>
        </li>
        <li>
          <span>Call{phone.call ? ` · ${phone.call.name}` : ""}</span>
          {#if !phone.call}
            <button class="wide" onclick={() => phoneSimulate("call")}>Ring</button>
          {:else if !phone.call.answered_at}
            <span class="row">
              <button class="wide" onclick={() => phoneCommand("decline")}>Decline</button>
              <button class="wide" onclick={() => phoneCommand("accept")}>Accept</button>
            </span>
          {:else}
            <button class="wide" onclick={() => phoneCommand("hangup")}>Hang up</button>
          {/if}
        </li>
        <li>
          <span>Keys</span>
          <span class="hint">[ / ] volume · m mute · n notify · c call</span>
        </li>
      </ul>
    {/if}
  </div>
</div>

<style>
  .settings :global(:where(button)) {
    min-width: 48px;
    height: 48px;
    padding: 0 var(--s-4);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--glass-border);
    border-radius: var(--r-pill);
    background: rgb(255 255 255 / 0.06);
    color: var(--text);
    font: inherit;
  }
  .settings :global(:where(button):active) {
    background: rgb(255 255 255 / 0.14);
  }
  .settings :global(:where(button):disabled) {
    opacity: 0.4;
  }
  .settings {
    height: 100%;
    display: grid;
    grid-template-columns: 240px 1fr;
  }
  aside {
    display: flex;
    flex-direction: column;
    gap: var(--s-1);
    padding: var(--s-6) var(--s-4);
    border-right: 1px solid var(--glass-border);
  }
  h1 {
    margin: 0 var(--s-3) var(--s-5);
    font-size: var(--fs-xl);
    font-weight: 650;
    letter-spacing: -0.02em;
  }
  h2 {
    margin: 0 0 var(--s-5);
    font-size: var(--fs-lg);
    font-weight: 600;
  }
  .tab {
    height: 56px;
    justify-content: flex-start;
    padding: 0 var(--s-4);
    border: none;
    border-radius: var(--r-md);
    background: none;
    color: var(--text-dim);
    text-align: left;
  }
  .tab.sel {
    background: var(--accent-soft);
    color: var(--text);
    box-shadow: inset 3px 0 0 var(--accent);
  }
  .panel {
    min-width: 0;
    overflow-y: auto;
    padding: var(--s-6) var(--s-7) calc(var(--s-6) + var(--mini, 0px));
  }
  ul {
    list-style: none;
    margin: var(--s-3) 0 0;
    padding: 0 var(--s-5);
    border-radius: var(--r-lg);
    box-shadow: none;
  }
  li {
    min-height: var(--touch);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-4);
  }
  li + li {
    border-top: 1px solid var(--glass-border);
  }
  li > span:first-child {
    display: grid;
  }
  small {
    font-size: var(--fs-xs);
    color: var(--text-dim);
  }
  ul + .caps {
    margin-top: var(--s-6);
  }
  .row {
    display: flex;
    align-items: center;
    gap: var(--s-3);
  }
  input[type="range"] {
    width: 240px;
    accent-color: var(--accent);
  }
  .val {
    min-width: 3.5em;
    text-align: right;
  }
  .text {
    width: 280px;
    height: 48px;
    padding: 0 var(--s-4);
    border: 1px solid var(--glass-border);
    border-radius: var(--r-md);
    background: rgb(255 255 255 / 0.06);
    color: var(--text);
    font: inherit;
  }
  .wide {
    min-width: 120px;
  }
  .danger {
    color: var(--danger);
  }
  .seg {
    display: inline-flex;
    padding: 3px;
    border-radius: var(--r-pill);
    background: rgb(255 255 255 / 0.06);
    border: 1px solid var(--glass-border);
  }
  .seg button {
    height: 42px;
    min-width: 72px;
    border: none;
    background: none;
    color: var(--text-dim);
  }
  .seg button.sel {
    background: var(--accent);
    color: #fff;
  }
  .toggle {
    position: relative;
    width: 64px;
    height: 36px;
    padding: 0;
    flex: none;
  }
  .toggle span {
    position: absolute;
    top: 3px;
    left: 3px;
    width: 28px;
    height: 28px;
    border-radius: var(--r-pill);
    background: var(--n-300);
    transition: transform var(--dur) var(--ease);
  }
  .toggle.on {
    background: var(--accent);
    border-color: var(--accent);
  }
  .toggle.on span {
    background: #fff;
    transform: translateX(28px);
  }
  .swatch {
    width: 44px;
    height: 44px;
    padding: 0;
    background: var(--c);
  }
  .swatch.sel {
    box-shadow:
      0 0 0 3px var(--bg),
      0 0 0 5px var(--c);
  }
  .eq {
    justify-content: space-around;
    padding: var(--s-4) 0;
  }
  .eq label {
    display: grid;
    justify-items: center;
    gap: var(--s-2);
  }
  .eq input {
    width: 40px;
    height: 160px;
    writing-mode: vertical-lr;
    direction: rtl;
  }
  .hint,
  .dim {
    font-size: var(--fs-sm);
    color: var(--text-dim);
  }
  .settings :global(.pill) {
    display: inline-flex;
    align-items: center;
    gap: var(--s-2);
    padding: var(--s-1) var(--s-3);
    border-radius: var(--r-pill);
    font-size: var(--fs-sm);
    font-weight: 500;
    color: var(--warn);
    background: var(--warn-soft);
  }
  .settings :global(.pill.ok) {
    color: var(--ok);
    background: var(--ok-soft);
  }
  .settings :global(.dot) {
    width: 8px;
    height: 8px;
    border-radius: var(--r-pill);
    background: currentColor;
  }
</style>
