use crate::api::{InputEvent, Key};
use std::future::Future;
use tokio::io::{stdin, AsyncBufReadExt, BufReader, Lines, Stdin};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

pub trait Input {
    fn next(&mut self) -> impl Future<Output = InputEvent> + Send;
}

/// Events from WebSocket clients, plus stdin: w/a/x/d = up/left/down/right,
/// empty line/s = select, q = back, +/- = knob.
pub struct MockInput {
    lines: Option<Lines<BufReader<Stdin>>>,
    tx: UnboundedSender<InputEvent>,
    rx: UnboundedReceiver<InputEvent>,
}

impl MockInput {
    pub fn new() -> Self {
        let (tx, rx) = unbounded_channel();
        Self { lines: Some(BufReader::new(stdin()).lines()), tx, rx }
    }

    pub fn sender(&self) -> UnboundedSender<InputEvent> {
        self.tx.clone()
    }
}

impl Input for MockInput {
    async fn next(&mut self) -> InputEvent {
        loop {
            let line = match &mut self.lines {
                Some(lines) => tokio::select! {
                    Some(ev) = self.rx.recv() => return ev,
                    line = lines.next_line() => line,
                },
                None => return self.rx.recv().await.unwrap(),
            };
            let Ok(Some(line)) = line else {
                self.lines = None;
                continue;
            };
            let key = match line.trim() {
                "w" | "up" => Key::Up,
                "x" | "down" => Key::Down,
                "a" | "left" => Key::Left,
                "d" | "right" => Key::Right,
                "" | "s" | "select" => Key::Select,
                "q" | "back" => Key::Back,
                "+" | "=" => return InputEvent::Knob(1),
                "-" => return InputEvent::Knob(-1),
                _ => continue,
            };
            let _ = self.tx.send(InputEvent::Release(key));
            return InputEvent::Press(key);
        }
    }
}
