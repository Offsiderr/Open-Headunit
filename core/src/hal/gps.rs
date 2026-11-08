use crate::api::GpsFix;
use std::future::Future;
use std::time::Duration;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::time::{interval, Interval};

pub trait Gps {
    fn next(&mut self) -> impl Future<Output = GpsFix> + Send;
}

/// Loops around Christchurch's four avenues: Bealey, Fitzgerald, Moorhouse, Deans/Harper.
const ROUTE: [(f64, f64); 30] = [
    (-43.5218, 172.6270),
    (-43.5216, 172.6320),
    (-43.5214, 172.6370),
    (-43.5213, 172.6420),
    (-43.5212, 172.6470),
    (-43.5212, 172.6510),
    (-43.5250, 172.6512),
    (-43.5290, 172.6513),
    (-43.5330, 172.6514),
    (-43.5370, 172.6515),
    (-43.5405, 172.6515),
    (-43.5407, 172.6460),
    (-43.5408, 172.6410),
    (-43.5408, 172.6360),
    (-43.5405, 172.6310),
    (-43.5400, 172.6270),
    (-43.5385, 172.6240),
    (-43.5365, 172.6215),
    (-43.5355, 172.6190),
    (-43.5348, 172.6165),
    (-43.5345, 172.6140),
    (-43.5340, 172.6110),
    (-43.5300, 172.6113),
    (-43.5265, 172.6117),
    (-43.5230, 172.6122),
    (-43.5205, 172.6128),
    (-43.5193, 172.6165),
    (-43.5192, 172.6205),
    (-43.5196, 172.6230),
    (-43.5205, 172.6250),
];
const SPEED_KMH: f64 = 50.0;
const EARTH_RADIUS_M: f64 = 6_371_000.0;

/// Loops the avenues until given a path to drive (a nav route), then follows it and parks at the end.
pub struct MockGps {
    tick: Interval,
    tx: UnboundedSender<Vec<(f64, f64)>>,
    rx: UnboundedReceiver<Vec<(f64, f64)>>,
    path: Vec<(f64, f64)>,
    looped: bool,
    seg: usize,
    along_m: f64,
    heading: f64,
}

impl MockGps {
    pub fn new() -> Self {
        let (tx, rx) = unbounded_channel();
        Self {
            tick: interval(Duration::from_secs(1)),
            tx,
            rx,
            path: ROUTE.to_vec(),
            looped: true,
            seg: 0,
            along_m: 0.0,
            heading: 0.0,
        }
    }

    pub fn driver(&self) -> UnboundedSender<Vec<(f64, f64)>> {
        self.tx.clone()
    }

    fn segment(&self, i: usize) -> ((f64, f64), (f64, f64)) {
        (self.path[i], self.path[(i + 1) % self.path.len()])
    }

    fn parked(&self) -> bool {
        !self.looped && self.seg + 1 >= self.path.len()
    }
}

impl Gps for MockGps {
    async fn next(&mut self) -> GpsFix {
        self.tick.tick().await;
        while let Ok(path) = self.rx.try_recv() {
            if !path.is_empty() {
                self.path = path;
                self.looped = false;
                self.seg = 0;
                self.along_m = 0.0;
            }
        }
        if self.parked() {
            let (lat, lon) = *self.path.last().unwrap();
            return GpsFix { lat, lon, heading_deg: self.heading, speed_kmh: 0.0, fix: true };
        }
        let (a, b) = self.segment(self.seg);
        let (north, east) = offset_m(a, b);
        let len = north.hypot(east);
        let f = if len > 0.0 { self.along_m / len } else { 0.0 };
        if len > 0.0 {
            self.heading = east.atan2(north).to_degrees().rem_euclid(360.0);
        }
        let fix = GpsFix {
            lat: a.0 + (b.0 - a.0) * f,
            lon: a.1 + (b.1 - a.1) * f,
            heading_deg: self.heading,
            speed_kmh: SPEED_KMH,
            fix: true,
        };
        self.along_m += SPEED_KMH / 3.6;
        while !self.parked() {
            let (a, b) = self.segment(self.seg);
            let (north, east) = offset_m(a, b);
            let len = north.hypot(east);
            if self.along_m < len {
                break;
            }
            self.along_m -= len;
            self.seg = (self.seg + 1) % self.path.len();
        }
        fix
    }
}

pub fn distance_m(a: (f64, f64), b: (f64, f64)) -> f64 {
    let (north, east) = offset_m(a, b);
    north.hypot(east)
}

fn offset_m(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    let north = (b.0 - a.0).to_radians() * EARTH_RADIUS_M;
    let east = (b.1 - a.1).to_radians() * EARTH_RADIUS_M * a.0.to_radians().cos();
    (north, east)
}
