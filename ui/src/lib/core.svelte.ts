import type {
  Call,
  CanFrame,
  Destinations,
  Envelope,
  GpsFix,
  InputEvent,
  Key,
  Launch,
  MediaCommand,
  MediaState,
  Msg,
  Notification,
  PhoneCommand,
  PhoneSimulate,
  Place,
  Progress,
  Route,
  Telemetry,
} from "../api";

export const core = $state({
  connected: false,
  kmh: 0,
  rpm: 0,
  telemetry: null as Telemetry | null,
  launch: null as Launch | null,
  media: null as MediaState | null,
  doors: { left: false, right: false },
});

export const gps = $state({ current: null as GpsFix | null });

export const nav = $state({
  query: "",
  results: [] as Place[],
  searching: false,
  destinations: { saved: [], recent: [] } as Destinations,
  route: null as Route | null,
  progress: null as Progress | null,
  arrived: null as Place | null,
  error: null as string | null,
});

const ARRIVED_MS = 8000;

export function navSearch(query: string) {
  nav.query = query;
  nav.error = null;
  nav.searching = !!query.trim();
  if (!nav.searching) nav.results = [];
  send({ topic: "nav.search", payload: query });
}

export function navigate(place: Place) {
  nav.error = null;
  nav.arrived = null;
  send({ topic: "nav.navigate", payload: place });
}

export const navCancel = () => send({ topic: "nav.cancel" });
export const navSave = (place: Place) => send({ topic: "nav.save", payload: place });

export type CanRow = CanFrame & { hz: number; changed: boolean[] };

export const can = $state({ rows: [] as CanRow[], fps: 0 });

let canPrev = new Map<number, CanFrame>();
let canTs = 0;

function canFrames(frames: CanFrame[], ts: number) {
  const dt = canTs ? (ts - canTs) / 1000 : 0;
  can.rows = frames.map((f) => {
    const p = canPrev.get(f.id);
    return {
      ...f,
      hz: p && dt > 0 ? (f.count - p.count) / dt : 0,
      changed: f.data.map((b, i) => !!p && p.data[i] !== b),
    };
  });
  can.fps = can.rows.reduce((n, r) => n + r.hz, 0);
  canPrev = new Map(frames.map((f) => [f.id, f]));
  canTs = ts;
}

export const phone = $state({ call: null as Call | null, toasts: [] as Notification[] });

const TOAST_MS = 6000;
const MAX_TOASTS = 3;

export function dismiss(id: number) {
  phone.toasts = phone.toasts.filter((t) => t.id !== id);
}

function toast(n: Notification) {
  phone.toasts = [n, ...phone.toasts].slice(0, MAX_TOASTS);
  setTimeout(() => dismiss(n.id), TOAST_MS);
}

// Local only until volume comes from core. `rev` bumps on every change so the overlay can react.
export const volume = $state({ level: 40, muted: false, rev: 0 });

export function setVolume(level: number, muted = false) {
  volume.level = Math.max(0, Math.min(100, Math.round(level)));
  volume.muted = muted;
  volume.rev++;
}

const inputListeners = new Set<(ev: InputEvent) => void>();

export function onInput(fn: (ev: InputEvent) => void) {
  inputListeners.add(fn);
  return () => {
    inputListeners.delete(fn);
  };
}

let ws: WebSocket;

function send(msg: Msg) {
  if (ws.readyState === WebSocket.OPEN) ws.send(JSON.stringify({ v: 1, ts: Date.now(), ...msg }));
}

export const mediaCommand = (cmd: MediaCommand) => send({ topic: "media.command", payload: cmd });
export const phoneCommand = (cmd: PhoneCommand) => send({ topic: "phone.command", payload: cmd });
export const phoneSimulate = (sim: PhoneSimulate) => send({ topic: "phone.simulate", payload: sim });

function connect() {
  ws = new WebSocket(`ws://${location.host}/ws`);
  ws.onopen = () => {
    core.connected = true;
    send({ topic: "nav.sync" });
  };
  ws.onclose = () => {
    core.connected = false;
    setTimeout(connect, 1000);
  };
  ws.onmessage = (e) => {
    const m: Envelope = JSON.parse(e.data);
    switch (m.topic) {
      case "vehicle.speed": core.kmh = m.payload.kmh; break;
      case "vehicle.rpm": core.rpm = m.payload.rpm; break;
      case "vehicle.telemetry": core.telemetry = m.payload; break;
      case "vehicle.launch": core.launch = m.payload; break;
      case "media.state": core.media = m.payload; break;
      case "gps.position": gps.current = m.payload; break;
      case "phone.notification": toast(m.payload); break;
      case "phone.call": phone.call = m.payload; break;
      case "can.frames": canFrames(m.payload, m.ts); break;
      case "nav.results":
        if (m.payload.query === nav.query) {
          nav.results = m.payload.places;
          nav.searching = false;
        }
        break;
      case "nav.destinations": nav.destinations = m.payload; break;
      case "nav.route":
        nav.route = m.payload;
        if (!m.payload) nav.progress = null;
        break;
      case "nav.progress": nav.progress = m.payload; break;
      case "nav.arrived": {
        const place = m.payload;
        nav.arrived = place;
        setTimeout(() => nav.arrived === place && (nav.arrived = null), ARRIVED_MS);
        break;
      }
      case "nav.error":
        nav.error = m.payload;
        nav.searching = false;
        break;
      case "input.event": for (const fn of inputListeners) fn(m.payload); break;
    }
  };
}

connect();

const keys: Record<string, Key> = {
  ArrowUp: "up",
  ArrowDown: "down",
  ArrowLeft: "left",
  ArrowRight: "right",
  Enter: "select",
  Escape: "back",
};
const knob: Record<string, number> = { "+": 1, "=": 1, "-": -1 };

// Debug: 1/2 toggle the left/right door until door state comes from the car.
const doorKeys: Record<string, "left" | "right"> = { "1": "left", "2": "right" };

// Debug: [ / ] step volume, m toggles mute.
const volumeKeys: Record<string, number> = { "[": -5, "]": 5 };

// Debug: n/c simulate a phone notification / incoming call until Bluetooth exists.
const phoneKeys: Record<string, PhoneSimulate> = { n: "notification", c: "call" };

const typing = (e: KeyboardEvent) => e.target instanceof HTMLInputElement;

addEventListener("keydown", (e) => {
  if (typing(e)) return;
  if (phoneKeys[e.key]) {
    if (!e.repeat) phoneSimulate(phoneKeys[e.key]);
  } else if (volumeKeys[e.key]) {
    setVolume(volume.level + volumeKeys[e.key]);
  } else if (e.key === "m") {
    if (!e.repeat) setVolume(volume.level, !volume.muted);
  } else if (doorKeys[e.key]) {
    if (!e.repeat) core.doors[doorKeys[e.key]] = !core.doors[doorKeys[e.key]];
  } else if (keys[e.key]) {
    if (!e.repeat) send({ topic: "input.event", payload: { press: keys[e.key] } });
  } else if (knob[e.key]) {
    send({ topic: "input.event", payload: { knob: knob[e.key] } });
  }
});
addEventListener("keyup", (e) => {
  if (!typing(e) && keys[e.key]) send({ topic: "input.event", payload: { release: keys[e.key] } });
});
