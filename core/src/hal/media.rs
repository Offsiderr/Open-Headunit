use crate::api::{MediaCommand, MediaState};
use std::future::Future;
use std::time::Duration;
use tokio::time::{interval, Interval};

pub trait Media {
    fn next(&mut self) -> impl Future<Output = MediaState> + Send;
    fn command(&mut self, cmd: MediaCommand) -> impl Future<Output = ()> + Send;
}

const TRACKS: [(&str, &str, u64, &str); 4] = [
    ("Floating", "Mock Artist", 185_000, "/art/floating.jpg"),
    ("Hurricane", "The Flat Fours", 214_000, "/art/hurricane.jpg"),
    ("Kids See Ghosts", "Synthetic Signals", 162_000, "/art/kids-see-ghosts.jpg"),
    ("Leave You", "Boxer Rumble", 198_000, "/art/leave-you.jpg"),
];
const STEP_MS: u64 = 500;
const RESTART_MS: u64 = 3_000;

pub struct MockMedia {
    tick: Interval,
    track: usize,
    position_ms: u64,
    playing: bool,
}

impl MockMedia {
    pub fn new() -> Self {
        Self { tick: interval(Duration::from_millis(STEP_MS)), track: 0, position_ms: 0, playing: true }
    }

    fn skip(&mut self, by: usize) {
        self.track = (self.track + by) % TRACKS.len();
        self.position_ms = 0;
    }
}

impl Media for MockMedia {
    async fn next(&mut self) -> MediaState {
        self.tick.tick().await;
        let (title, artist, duration_ms, art) = TRACKS[self.track];
        let state = MediaState {
            playing: self.playing,
            title: title.into(),
            artist: artist.into(),
            position_ms: self.position_ms,
            duration_ms,
            art: Some(art.into()),
        };
        if self.playing {
            self.position_ms += STEP_MS;
            if self.position_ms > duration_ms {
                self.skip(1);
            }
        }
        state
    }

    async fn command(&mut self, cmd: MediaCommand) {
        match cmd {
            MediaCommand::Toggle => self.playing = !self.playing,
            MediaCommand::Next => self.skip(1),
            MediaCommand::Prev if self.position_ms > RESTART_MS => self.position_ms = 0,
            MediaCommand::Prev => self.skip(TRACKS.len() - 1),
        }
        self.tick.reset_immediately();
    }
}
