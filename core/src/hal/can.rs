use std::collections::VecDeque;
use std::future::Future;
use std::time::Duration;
use tokio::time::{interval, Interval};

pub struct Frame {
    pub id: u32,
    pub data: Vec<u8>,
}

pub trait Can {
    fn next(&mut self) -> impl Future<Output = Frame> + Send;
}

/// Synthetic ZC6 BRZ powertrain bus: 50 Hz chassis frames, 10 Hz body/engine frames.
/// Layouts follow community 86/BRZ CAN notes; byte 6 is a rolling counter, byte 7 a Toyota-style checksum.
pub struct MockCan {
    tick: Interval,
    n: u64,
    queue: VecDeque<Frame>,
}

impl MockCan {
    pub fn new() -> Self {
        Self { tick: interval(Duration::from_millis(20)), n: 0, queue: VecDeque::new() }
    }

    fn fill(&mut self) {
        let t = self.n as f64 * 0.02;
        let kmh = 60.0 + 40.0 * t.sin();
        let rpm = 800.0 + kmh * 45.0;
        let accel = t.cos();
        let throttle = (accel * 80.0).clamp(0.0, 100.0);
        let brake = (-accel * 120.0).clamp(0.0, 255.0);
        let steer = 30.0 * (t * 0.37).sin();
        let coolant = (20.0 + t * 0.5).min(90.0);
        let oil = (20.0 + t * 0.3).min(95.0);
        let gear = (1.0 + kmh / 22.0).clamp(1.0, 6.0) as u8;
        let speed = ((kmh / 0.015625) as u16).to_le_bytes();
        let ctr = (self.n & 0x0f) as u8;

        let steer = ((steer * 10.0) as i16).to_le_bytes();
        self.push(0x0d0, [steer[0], steer[1], 0, 0, 0, 0, ctr, 0]);
        self.push(0x0d1, [speed[0], speed[1], brake as u8, 0, 0, 0, ctr, 0]);
        let wheel = |k: f64| (((kmh * k) / 0.015625) as u16).to_le_bytes();
        let (fl, fr, rl, rr) = (wheel(1.0 + steer_bias(t)), wheel(1.0 - steer_bias(t)), wheel(1.0), wheel(1.0));
        self.push(0x0d4, [fl[0], fl[1], fr[0], fr[1], rl[0], rl[1], rr[0], rr[1]]);
        let rpm = ((rpm as u16) & 0x3fff).to_le_bytes();
        self.push(0x140, [(throttle * 2.55) as u8, 0, rpm[0], rpm[1], 0, 0, ctr, 0]);
        if self.n % 5 == 0 {
            self.push(0x360, [0, 0, (oil + 40.0) as u8, (coolant + 40.0) as u8, 0, 0, ctr, 0]);
            self.push(0x361, [gear, 0, 0, 0, 0, 0, ctr, 0]);
            self.push(0x375, [0, 0, 0, 0, 0, 0, ctr, 0]);
        }
        self.n += 1;
    }

    fn push(&mut self, id: u32, mut data: [u8; 8]) {
        if id != 0x0d4 {
            data[7] = checksum(id, &data[..7]);
        }
        self.queue.push_back(Frame { id, data: data.to_vec() });
    }
}

fn steer_bias(t: f64) -> f64 {
    0.01 * (t * 0.37).sin()
}

fn checksum(id: u32, data: &[u8]) -> u8 {
    let sum = (id & 0xff) + (id >> 8) + data.len() as u32 + 1 + data.iter().map(|&b| b as u32).sum::<u32>();
    sum as u8
}

impl Can for MockCan {
    async fn next(&mut self) -> Frame {
        loop {
            if let Some(f) = self.queue.pop_front() {
                return f;
            }
            self.tick.tick().await;
            self.fill();
        }
    }
}
