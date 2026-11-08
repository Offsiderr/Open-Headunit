use crate::api::Telemetry;
use std::future::Future;
use std::time::Duration;
use tokio::time::{interval, Interval};

pub struct Sample {
    pub kmh: f64,
    pub rpm: f64,
    pub telemetry: Telemetry,
}

pub trait Vehicle {
    fn next(&mut self) -> impl Future<Output = Sample> + Send;
}

const DT: f64 = 0.05;
const RATIOS: [f64; 6] = [3.626, 2.188, 1.541, 1.213, 1.0, 0.767];
/// Engine rpm per km/h per unit gear ratio (4.3 final drive, 0.32 m tyre radius).
const RPM_PER_KMH: f64 = 35.6;
const IDLE_RPM: f64 = 800.0;
const SHIFT_RPM: f64 = 7200.0;
const SHIFT_S: f64 = 0.2;

#[derive(PartialEq)]
enum Phase {
    Idle,
    Launch,
    Cruise,
    Brake,
}

/// Loops a drive cycle (idle, full-throttle launch, cruise, brake) so 0-100 runs happen.
pub struct MockVehicle {
    tick: Interval,
    t: f64,
    phase: Phase,
    phase_t: f64,
    runs: u32,
    kmh: f64,
    gear: u8,
    shift_t: f64,
    fuel: f64,
}

impl MockVehicle {
    pub fn new() -> Self {
        Self {
            tick: interval(Duration::from_millis(50)),
            t: 0.0,
            phase: Phase::Idle,
            phase_t: 0.0,
            runs: 0,
            kmh: 0.0,
            gear: 0,
            shift_t: 0.0,
            fuel: 72.0,
        }
    }

    fn set(&mut self, phase: Phase) {
        self.phase = phase;
        self.phase_t = 0.0;
    }

    fn gear_rpm(&self) -> f64 {
        self.kmh * RPM_PER_KMH * RATIOS[self.gear.max(1) as usize - 1]
    }

    fn step(&mut self) {
        self.t += DT;
        self.phase_t += DT;
        self.shift_t = (self.shift_t - DT).max(0.0);
        match self.phase {
            Phase::Idle => {
                self.gear = 0;
                if self.phase_t > 4.0 {
                    self.runs += 1;
                    self.gear = 1;
                    self.set(Phase::Launch);
                }
            }
            Phase::Launch => {
                // Pseudo-random grip per run so times vary between ~6.2 and ~6.9 s.
                let grip = 19.0 + 2.5 * (self.runs as f64 * 12.9898).sin().abs();
                if self.shift_t == 0.0 {
                    self.kmh += grip * (1.0 - self.kmh / 230.0) * DT;
                }
                if self.gear_rpm() > SHIFT_RPM && self.gear < 6 {
                    self.gear += 1;
                    self.shift_t = SHIFT_S;
                }
                if self.kmh >= 125.0 {
                    self.set(Phase::Cruise);
                }
            }
            Phase::Cruise => {
                self.kmh = 110.0 + 15.0 * (self.phase_t * 0.8).cos();
                if self.gear < 6 && self.gear_rpm() > 3200.0 {
                    self.gear += 1;
                }
                if self.phase_t > 6.0 {
                    self.set(Phase::Brake);
                }
            }
            Phase::Brake => {
                self.kmh = (self.kmh - 28.0 * DT).max(0.0);
                if self.gear > 1 && self.gear_rpm() < 1600.0 {
                    self.gear -= 1;
                }
                if self.kmh < 8.0 {
                    self.gear = 0;
                }
                if self.kmh == 0.0 {
                    self.set(Phase::Idle);
                }
            }
        }
    }

    fn rpm(&self) -> f64 {
        let idle = IDLE_RPM + 15.0 * (self.t * 7.0).sin();
        if self.gear == 0 {
            return idle;
        }
        let rpm = self.gear_rpm();
        if self.phase == Phase::Launch && self.gear == 1 {
            rpm.max(3500.0)
        } else {
            rpm.max(idle)
        }
    }
}

impl Vehicle for MockVehicle {
    async fn next(&mut self) -> Sample {
        self.tick.tick().await;
        self.step();
        let rpm = self.rpm();
        let load = rpm / SHIFT_RPM;
        self.fuel = (self.fuel - load * 0.01 * DT).max(0.0);
        let telemetry = Telemetry {
            gear: self.gear,
            coolant_c: 90.0 - 70.0 * (-self.t / 45.0).exp() + 2.0 * load,
            oil_c: 98.0 - 78.0 * (-self.t / 90.0).exp() + 4.0 * load,
            intake_c: 24.0 + 6.0 * (-self.kmh / 60.0).exp() + (self.t * 0.05).sin(),
            battery_v: 14.1 + 0.08 * (self.t * 0.7).sin() - 0.1 * load,
            fuel_pct: self.fuel,
        };
        Sample { kmh: self.kmh, rpm, telemetry }
    }
}
