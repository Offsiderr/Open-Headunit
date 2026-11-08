export type Key = "up" | "down" | "left" | "right" | "select" | "back";

export type InputEvent = { press: Key } | { release: Key } | { knob: number };

export interface MediaState {
  playing: boolean;
  title: string;
  artist: string;
  position_ms: number;
  duration_ms: number;
  art: string | null;
}

export type MediaCommand = "prev" | "toggle" | "next";

export interface GpsFix {
  lat: number;
  lon: number;
  heading_deg: number;
  speed_kmh: number;
  fix: boolean;
}

export interface Notification {
  id: number;
  app: string;
  title: string;
  body: string;
}

export interface Call {
  name: string;
  number: string;
  answered_at: number | null;
}

export interface Telemetry {
  gear: number;
  coolant_c: number;
  oil_c: number;
  intake_c: number;
  battery_v: number;
  fuel_pct: number;
}

export interface Launch {
  running_ms: number | null;
  last_ms: number | null;
  best_ms: number | null;
}

export type PhoneCommand = "accept" | "decline" | "hangup";

export type PhoneSimulate = "notification" | "call";

export interface Place {
  name: string;
  address: string;
  lat: number;
  lon: number;
}

export interface Destinations {
  saved: Place[];
  recent: Place[];
}

export interface Step {
  instruction: string;
  modifier: string;
  distance_m: number;
  lat: number;
  lon: number;
}

export interface Route {
  destination: Place;
  distance_m: number;
  duration_s: number;
  /** [lat, lon] pairs. */
  geometry: [number, number][];
  steps: Step[];
}

export interface Progress {
  step: number;
  step_distance_m: number;
  remaining_m: number;
  remaining_s: number;
}

export interface CanFrame {
  id: number;
  data: number[];
  count: number;
}

export type Msg =
  | { topic: "vehicle.speed"; payload: { kmh: number } }
  | { topic: "vehicle.rpm"; payload: { rpm: number } }
  | { topic: "vehicle.telemetry"; payload: Telemetry }
  | { topic: "vehicle.launch"; payload: Launch }
  | { topic: "media.state"; payload: MediaState }
  | { topic: "input.event"; payload: InputEvent }
  | { topic: "gps.position"; payload: GpsFix }
  | { topic: "media.command"; payload: MediaCommand }
  | { topic: "phone.notification"; payload: Notification }
  | { topic: "phone.call"; payload: Call | null }
  | { topic: "phone.command"; payload: PhoneCommand }
  | { topic: "phone.simulate"; payload: PhoneSimulate }
  | { topic: "can.frames"; payload: CanFrame[] }
  | { topic: "nav.search"; payload: string }
  | { topic: "nav.results"; payload: { query: string; places: Place[] } }
  | { topic: "nav.navigate"; payload: Place }
  | { topic: "nav.cancel" }
  | { topic: "nav.save"; payload: Place }
  | { topic: "nav.sync" }
  | { topic: "nav.destinations"; payload: Destinations }
  | { topic: "nav.route"; payload: Route | null }
  | { topic: "nav.progress"; payload: Progress }
  | { topic: "nav.arrived"; payload: Place }
  | { topic: "nav.error"; payload: string };

export type Envelope = { v: number; ts: number } & Msg;
