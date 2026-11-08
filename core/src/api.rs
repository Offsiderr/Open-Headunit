use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

pub const VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
#[serde(tag = "topic", content = "payload")]
pub enum Msg {
    #[serde(rename = "vehicle.speed")]
    VehicleSpeed { kmh: f64 },
    #[serde(rename = "vehicle.rpm")]
    VehicleRpm { rpm: f64 },
    #[serde(rename = "vehicle.telemetry")]
    VehicleTelemetry(Telemetry),
    #[serde(rename = "vehicle.launch")]
    VehicleLaunch(Launch),
    #[serde(rename = "media.state")]
    MediaState(MediaState),
    #[serde(rename = "input.event")]
    InputEvent(InputEvent),
    #[serde(rename = "gps.position")]
    GpsPosition(GpsFix),
    #[serde(rename = "media.command")]
    MediaCommand(MediaCommand),
    #[serde(rename = "phone.notification")]
    PhoneNotification(Notification),
    #[serde(rename = "phone.call")]
    PhoneCall(Option<Call>),
    #[serde(rename = "phone.command")]
    PhoneCommand(PhoneCommand),
    #[serde(rename = "phone.simulate")]
    PhoneSimulate(PhoneSimulate),
    #[serde(rename = "nav.search")]
    NavSearch(String),
    #[serde(rename = "nav.results")]
    NavResults { query: String, places: Vec<Place> },
    #[serde(rename = "nav.navigate")]
    NavNavigate(Place),
    #[serde(rename = "nav.cancel")]
    NavCancel,
    #[serde(rename = "nav.save")]
    NavSave(Place),
    #[serde(rename = "nav.sync")]
    NavSync,
    #[serde(rename = "nav.destinations")]
    NavDestinations(Destinations),
    #[serde(rename = "nav.route")]
    NavRoute(Option<Route>),
    #[serde(rename = "nav.progress")]
    NavProgress(Progress),
    #[serde(rename = "nav.arrived")]
    NavArrived(Place),
    #[serde(rename = "nav.error")]
    NavError(String),
    #[serde(rename = "can.frames")]
    CanFrames(Vec<CanFrame>),
}

#[derive(Serialize, Deserialize)]
pub struct CanFrame {
    pub id: u32,
    pub data: Vec<u8>,
    pub count: u64,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Telemetry {
    /// 0 = neutral.
    pub gear: u8,
    pub coolant_c: f64,
    pub oil_c: f64,
    pub intake_c: f64,
    pub battery_v: f64,
    pub fuel_pct: f64,
}

/// 0-100 km/h timing. `running_ms` is set while a run is in progress.
#[derive(Serialize, Deserialize, Clone, Copy, Default)]
pub struct Launch {
    pub running_ms: Option<u64>,
    pub last_ms: Option<u64>,
    pub best_ms: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Place {
    pub name: String,
    pub address: String,
    pub lat: f64,
    pub lon: f64,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Destinations {
    pub saved: Vec<Place>,
    pub recent: Vec<Place>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Route {
    pub destination: Place,
    pub distance_m: f64,
    pub duration_s: f64,
    /// (lat, lon) pairs.
    pub geometry: Vec<(f64, f64)>,
    pub steps: Vec<Step>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Step {
    pub instruction: String,
    pub modifier: String,
    pub distance_m: f64,
    pub lat: f64,
    pub lon: f64,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Progress {
    /// Index of the upcoming step in `Route::steps`.
    pub step: usize,
    pub step_distance_m: f64,
    pub remaining_m: f64,
    pub remaining_s: f64,
}

#[derive(Serialize, Deserialize)]
pub struct Notification {
    pub id: u64,
    pub app: String,
    pub title: String,
    pub body: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Call {
    pub name: String,
    pub number: String,
    pub answered_at: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PhoneCommand {
    Accept,
    Decline,
    Hangup,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PhoneSimulate {
    Notification,
    Call,
}

#[derive(Serialize, Deserialize)]
pub struct GpsFix {
    pub lat: f64,
    pub lon: f64,
    pub heading_deg: f64,
    pub speed_kmh: f64,
    pub fix: bool,
}

#[derive(Serialize, Deserialize)]
pub struct MediaState {
    pub playing: bool,
    pub title: String,
    pub artist: String,
    pub position_ms: u64,
    pub duration_ms: u64,
    pub art: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MediaCommand {
    Prev,
    Toggle,
    Next,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Select,
    Back,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum InputEvent {
    Press(Key),
    Release(Key),
    Knob(i8),
}

#[derive(Serialize, Deserialize)]
struct Envelope {
    v: u32,
    ts: u64,
    #[serde(flatten)]
    msg: Msg,
}

pub fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}

impl Msg {
    pub fn into_json(self) -> String {
        serde_json::to_string(&Envelope { v: VERSION, ts: now_ms(), msg: self }).unwrap()
    }

    pub fn from_json(s: &str) -> serde_json::Result<Self> {
        serde_json::from_str::<Envelope>(s).map(|e| e.msg)
    }
}
