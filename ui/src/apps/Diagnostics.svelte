<script lang="ts">
  import { can, core, type CanRow } from "../lib/core.svelte";
  import { speedUnit, tempUnit, toSpeed, toTemp } from "../lib/settings.svelte";

  const NAMES: Record<number, string> = {
    0x0d0: "Steering",
    0x0d1: "Speed / brake",
    0x0d4: "Wheel speeds",
    0x140: "Throttle / RPM",
    0x360: "Engine temps",
    0x361: "Gear",
    0x375: "Doors",
  };

  let frozen = $state<CanRow[] | null>(null);
  let filter = $state("");
  const rows = $derived(frozen ?? can.rows);
  const shown = $derived(
    rows.filter((r) => !filter || hex(r.id, 3).includes(filter.toUpperCase()) || NAMES[r.id]?.toLowerCase().includes(filter.toLowerCase())),
  );

  const hex = (n: number, w: number) => n.toString(16).toUpperCase().padStart(w, "0");
  const u16 = (d: number[], i: number) => d[i] | (d[i + 1] << 8);
  const i16 = (d: number[], i: number) => (u16(d, i) << 16) >> 16;

  const signals = $derived.by(() => {
    const d = (id: number) => rows.find((r) => r.id === id)?.data;
    const out: [string, string][] = [];
    const sp = (raw: number) => toSpeed(raw * 0.015625).toFixed(1);
    const t = (raw: number) => `${toTemp(raw - 40).toFixed(0)} ${tempUnit()}`;
    let f: number[] | undefined;
    if ((f = d(0x0d1))) out.push(["Speed", `${sp(u16(f, 0))} ${speedUnit()}`], ["Brake", `${Math.round((f[2] / 255) * 100)} %`]);
    if ((f = d(0x140))) out.push(["RPM", `${u16(f, 2) & 0x3fff}`], ["Throttle", `${Math.round(f[0] / 2.55)} %`]);
    if ((f = d(0x0d0))) out.push(["Steering", `${(i16(f, 0) / 10).toFixed(1)}°`]);
    if ((f = d(0x361))) out.push(["Gear", f[0] ? `${f[0]}` : "N"]);
    if ((f = d(0x360))) out.push(["Coolant", t(f[3])], ["Oil", t(f[2])]);
    if ((f = d(0x0d4))) out.push(["Wheels", [0, 2, 4, 6].map((i) => sp(u16(f!, i))).join(" · ")]);
    if ((f = d(0x375))) out.push(["Doors", f[2] ? "Open" : "Closed"]);
    return out;
  });
</script>

<div class="bar">
  <span class="pill" class:ok={core.connected && rows.length > 0}>
    <span class="dot"></span>{frozen ? "Paused" : rows.length ? "Live" : "No data"}
  </span>
  <span class="stat num">{rows.length} IDs · {Math.round(can.fps)} frames/s</span>
  <input placeholder="Filter ID or name" bind:value={filter} />
  <button onclick={() => (frozen = frozen ? null : $state.snapshot(can.rows))}>{frozen ? "Resume" : "Pause"}</button>
</div>

<div class="signals">
  {#each signals as [k, v]}
    <div class="glass" class:span={k === "Wheels"}><span class="caps">{k}</span><b class="num">{v}</b></div>
  {/each}
</div>

<div class="table glass">
  <div class="row head caps"><span>ID</span><span>Name</span><span>DLC</span><span>Data</span><span>Hz</span><span>Count</span></div>
  {#each shown as r (r.id)}
    <div class="row num">
      <span class="id">{hex(r.id, 3)}</span>
      <span class="name">{NAMES[r.id] ?? "—"}</span>
      <span>{r.data.length}</span>
      <span class="bytes">
        {#each r.data as b, i}<span class:hot={!frozen && r.changed[i]}>{hex(b, 2)}</span>{/each}
      </span>
      <span>{Math.round(r.hz)}</span>
      <span>{r.count}</span>
    </div>
  {:else}
    <div class="empty">Waiting for CAN frames…</div>
  {/each}
</div>
<p class="hint">Decoding follows community ZC6 86/BRZ notes and is approximate. Verify against your car.</p>

<style>
  .bar {
    display: flex;
    align-items: center;
    gap: var(--s-4);
  }
  .stat {
    color: var(--text-dim);
    font-size: var(--fs-sm);
  }
  input {
    flex: 1;
    min-width: 0;
    height: 48px;
    padding: 0 var(--s-4);
    border: 1px solid var(--glass-border);
    border-radius: var(--r-pill);
    background: rgb(255 255 255 / 0.06);
    color: var(--text);
    font: inherit;
  }
  .signals {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: var(--s-3);
    margin: var(--s-5) 0;
  }
  .signals div {
    display: grid;
    gap: var(--s-1);
    padding: var(--s-3) var(--s-4);
    border-radius: var(--r-md);
    box-shadow: none;
  }
  .signals .span {
    grid-column: span 2;
  }
  .signals b {
    font-size: var(--fs-lg);
    font-weight: 600;
    white-space: nowrap;
  }
  .table {
    padding: var(--s-2) var(--s-5);
    border-radius: var(--r-lg);
    box-shadow: none;
  }
  .row {
    display: grid;
    grid-template-columns: 64px 1fr 48px 300px 56px 80px;
    align-items: center;
    gap: var(--s-3);
    min-height: 44px;
    font-size: var(--fs-sm);
  }
  .row + .row {
    border-top: 1px solid var(--glass-border);
  }
  .row > :nth-child(n + 5) {
    text-align: right;
  }
  .head {
    min-height: 36px;
  }
  .id {
    color: var(--accent-hi);
    font-weight: 600;
    font-family: ui-monospace, monospace;
  }
  .name {
    color: var(--text-dim);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .bytes {
    display: flex;
    gap: 6px;
    font-family: ui-monospace, monospace;
  }
  .bytes span {
    width: 2.2ch;
    padding: 2px 4px;
    border-radius: 4px;
    text-align: center;
    transition: background var(--dur-slow) var(--ease);
  }
  .bytes .hot {
    background: var(--accent-soft);
    color: var(--accent-hi);
    transition: none;
  }
  .empty {
    padding: var(--s-5) 0;
    color: var(--text-dim);
    text-align: center;
  }
  .hint {
    margin: var(--s-3) 0 0;
    font-size: var(--fs-xs);
    color: var(--text-faint);
  }
</style>
