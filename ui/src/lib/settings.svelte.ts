import { volume } from "./core.svelte";

export const EQ_BANDS = ["60", "250", "1k", "4k", "12k"];

export const EQ_PRESETS: Record<string, number[]> = {
  Flat: [0, 0, 0, 0, 0],
  Bass: [6, 3, 0, 0, 1],
  Rock: [4, 1, -2, 2, 4],
  Vocal: [-2, 0, 3, 3, 0],
  Highway: [3, 1, 0, 2, 3],
};

export const ACCENTS: Record<string, [string, string]> = {
  Blue: ["#2a6fdb", "#4d8bf0"],
  Red: ["#d4343b", "#ef5a60"],
  Orange: ["#e0731f", "#f5913f"],
  Green: ["#25a35f", "#3fc47c"],
  Violet: ["#7a4fd6", "#9a73ef"],
};

export type Device = { name: string; addr: string; connected: boolean };

const DEFAULTS = {
  volume: 40,
  startupVolume: 50,
  eq: [...EQ_PRESETS.Flat],
  eqPreset: "Flat",
  balance: 0,
  fade: 0,
  loudness: false,
  speedVolume: "off" as "off" | "low" | "med" | "high",
  brightness: 100,
  nightBrightness: 40,
  nightMode: "auto" as "off" | "auto" | "on",
  accent: "Blue",
  solid: false,
  bluetooth: true,
  btName: "BRZ Headunit",
  discoverable: false,
  autoConnect: true,
  devices: [
    { name: "Pixel 8", addr: "4C:1A:2B:7E:90:11", connected: true },
    { name: "iPad", addr: "A8:5C:2C:13:4F:02", connected: false },
  ] as Device[],
  speedUnit: "kmh" as "kmh" | "mph",
  tempUnit: "c" as "c" | "f",
  clock24: true,
  clockSeconds: false,
};

const KEY = "settings";

function load(): Partial<typeof DEFAULTS> {
  try {
    return JSON.parse(localStorage.getItem(KEY) ?? "{}");
  } catch {
    return {};
  }
}

export const settings = $state({ ...structuredClone(DEFAULTS), ...load() });

volume.level = Math.min(settings.volume, settings.startupVolume);

export function resetSettings() {
  Object.assign(settings, structuredClone(DEFAULTS));
}

export const clock = $state({ now: new Date() });
setInterval(() => (clock.now = new Date()), 1000);

const isNight = (d: Date) => d.getHours() >= 19 || d.getHours() * 60 + d.getMinutes() < 390;

export const night = () => settings.nightMode === "on" || (settings.nightMode === "auto" && isNight(clock.now));

export const brightness = () => (night() ? Math.min(settings.brightness, settings.nightBrightness) : settings.brightness);

export const speedUnit = () => (settings.speedUnit === "mph" ? "mph" : "km/h");
export const toSpeed = (kmh: number) => (settings.speedUnit === "mph" ? kmh / 1.609344 : kmh);
export const tempUnit = () => (settings.tempUnit === "f" ? "°F" : "°C");
export const toTemp = (c: number) => (settings.tempUnit === "f" ? (c * 9) / 5 + 32 : c);

export function formatTime(d: Date) {
  const h = d.getHours();
  const mm = String(d.getMinutes()).padStart(2, "0");
  const ss = settings.clockSeconds ? `:${String(d.getSeconds()).padStart(2, "0")}` : "";
  if (settings.clock24) return `${String(h).padStart(2, "0")}:${mm}${ss}`;
  return `${h % 12 || 12}:${mm}${ss} ${h < 12 ? "AM" : "PM"}`;
}

$effect.root(() => {
  $effect(() => {
    settings.volume = volume.level;
  });
  $effect(() => {
    const json = JSON.stringify(settings);
    try {
      localStorage.setItem(KEY, json);
    } catch {}
  });
  $effect(() => {
    const root = document.documentElement.style;
    const [accent, hi] = ACCENTS[settings.accent] ?? ACCENTS.Blue;
    root.setProperty("--accent", accent);
    root.setProperty("--accent-hi", hi);
    root.setProperty("--accent-soft", `color-mix(in srgb, ${accent} 18%, transparent)`);
    root.setProperty("--glass", settings.solid ? "solid" : "glass");
  });
});
