export const GAUGES = {
  launch: "0–100",
  coolant: "Coolant",
  oil: "Oil",
  intake: "Intake",
  battery: "Battery",
  fuel: "Fuel",
};

export type Gauge = keyof typeof GAUGES;

const KEY = "gauges";
const DEFAULTS: Record<Gauge, boolean> = {
  launch: true,
  coolant: true,
  oil: true,
  intake: false,
  battery: true,
  fuel: true,
};

function load(): Partial<Record<Gauge, boolean>> {
  try {
    return JSON.parse(localStorage.getItem(KEY) ?? "{}");
  } catch {
    return {};
  }
}

export const gauges = $state({ ...DEFAULTS, ...load() });

$effect.root(() => {
  $effect(() => {
    const json = JSON.stringify(gauges);
    try {
      localStorage.setItem(KEY, json);
    } catch {}
  });
});
