use crate::api::Launch;
use std::time::{Duration, Instant};

const STOPPED_KMH: f64 = 0.5;
const TARGET_KMH: f64 = 100.0;
const TIMEOUT: Duration = Duration::from_secs(30);

/// Times 0-100 km/h runs from a standstill, interpolating the exact crossing between samples.
pub struct LaunchTimer {
    state: Launch,
    start: Option<Instant>,
    armed: bool,
    prev: (Instant, f64),
}

impl LaunchTimer {
    pub fn new() -> Self {
        Self { state: Launch::default(), start: None, armed: false, prev: (Instant::now(), 0.0) }
    }

    pub fn update(&mut self, now: Instant, kmh: f64) -> Launch {
        let (prev_t, prev_kmh) = std::mem::replace(&mut self.prev, (now, kmh));
        if kmh < STOPPED_KMH {
            self.start = None;
            self.armed = true;
        } else if self.armed {
            self.start = Some(prev_t);
            self.armed = false;
        }
        if let Some(start) = self.start {
            if kmh >= TARGET_KMH {
                let frac = ((TARGET_KMH - prev_kmh) / (kmh - prev_kmh)).clamp(0.0, 1.0);
                let ms = (prev_t + (now - prev_t).mul_f64(frac) - start).as_millis() as u64;
                self.state.last_ms = Some(ms);
                self.state.best_ms = Some(self.state.best_ms.map_or(ms, |b| b.min(ms)));
                self.start = None;
            } else if now - start > TIMEOUT {
                self.start = None;
            }
        }
        self.state.running_ms = self.start.map(|s| (now - s).as_millis() as u64);
        self.state
    }
}
