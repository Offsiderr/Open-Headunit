use crate::api::{now_ms, Call, Notification, PhoneCommand, PhoneSimulate};
use std::future::Future;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

pub enum PhoneEvent {
    Notification(Notification),
    Call(Option<Call>),
}

pub trait Phone {
    fn next(&mut self) -> impl Future<Output = PhoneEvent> + Send;
    fn command(&mut self, cmd: PhoneCommand) -> impl Future<Output = ()> + Send;
}

const NOTIFICATIONS: [(&str, &str, &str); 4] = [
    ("Messages", "Sam", "On my way, be there in 10"),
    ("WhatsApp", "Track Day Crew", "Gates open at 7, don't be late"),
    ("Calendar", "Service booking", "BRZ oil change tomorrow at 9:00"),
    ("Messages", "Mum", "Drive safe x"),
];
const CALLERS: [(&str, &str); 3] = [("Sam", "+64 21 555 0142"), ("Mum", "+64 27 555 0199"), ("Unknown", "+64 9 555 0100")];

/// Events come from `phone.simulate` messages until a Bluetooth driver exists.
pub struct MockPhone {
    tx: UnboundedSender<PhoneSimulate>,
    rx: UnboundedReceiver<PhoneSimulate>,
    call: Option<Call>,
    changed: bool,
    count: usize,
}

impl MockPhone {
    pub fn new() -> Self {
        let (tx, rx) = unbounded_channel();
        Self { tx, rx, call: None, changed: false, count: 0 }
    }

    pub fn simulator(&self) -> UnboundedSender<PhoneSimulate> {
        self.tx.clone()
    }
}

impl Phone for MockPhone {
    async fn next(&mut self) -> PhoneEvent {
        loop {
            if self.changed {
                self.changed = false;
                return PhoneEvent::Call(self.call.clone());
            }
            let sim = self.rx.recv().await.unwrap();
            self.count += 1;
            match sim {
                PhoneSimulate::Notification => {
                    let (app, title, body) = NOTIFICATIONS[self.count % NOTIFICATIONS.len()];
                    return PhoneEvent::Notification(Notification {
                        id: self.count as u64,
                        app: app.into(),
                        title: title.into(),
                        body: body.into(),
                    });
                }
                PhoneSimulate::Call if self.call.is_none() => {
                    let (name, number) = CALLERS[self.count % CALLERS.len()];
                    self.call = Some(Call { name: name.into(), number: number.into(), answered_at: None });
                    self.changed = true;
                }
                PhoneSimulate::Call => {}
            }
        }
    }

    async fn command(&mut self, cmd: PhoneCommand) {
        let Some(call) = &mut self.call else { return };
        match cmd {
            PhoneCommand::Accept if call.answered_at.is_none() => call.answered_at = Some(now_ms()),
            PhoneCommand::Accept => return,
            PhoneCommand::Decline | PhoneCommand::Hangup => self.call = None,
        }
        self.changed = true;
    }
}
