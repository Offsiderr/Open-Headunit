mod api;
mod hal;
mod launch;

use api::{CanFrame, Destinations, InputEvent, MediaCommand, Msg, PhoneCommand, PhoneSimulate, Progress, Route};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use hal::{
    can::{Can, MockCan},
    gps::{distance_m, Gps, MockGps},
    input::{Input, MockInput},
    media::{Media, MockMedia},
    nav::{MockNav, Nav, OsmNav},
    phone::{MockPhone, Phone, PhoneEvent},
    vehicle::{MockVehicle, Vehicle},
};
use std::collections::BTreeMap;
use std::time::Duration;
use launch::LaunchTimer;
use std::time::Instant;
use tokio::sync::{
    broadcast::{self, error::RecvError, Sender},
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    watch,
};
use tower_http::services::ServeDir;

#[derive(Clone)]
struct AppState {
    tx: Sender<String>,
    input: UnboundedSender<InputEvent>,
    media: UnboundedSender<MediaCommand>,
    phone: UnboundedSender<PhoneCommand>,
    simulate: UnboundedSender<PhoneSimulate>,
    nav: UnboundedSender<Msg>,
}

const DESTINATIONS_FILE: &str = "destinations.json";
const MAX_RECENT: usize = 8;
const REACHED_M: f64 = 30.0;

#[tokio::main]
async fn main() {
    let (tx, _) = broadcast::channel::<String>(256);
    let input = MockInput::new();
    let (media_tx, media_rx) = unbounded_channel();
    let phone = MockPhone::new();
    let (phone_tx, phone_rx) = unbounded_channel();
    let (nav_tx, nav_rx) = unbounded_channel();
    let gps = MockGps::new();
    let drive = gps.driver();
    let (pos_tx, pos_rx) = watch::channel(None);
    let state = AppState {
        tx: tx.clone(),
        input: input.sender(),
        media: media_tx,
        phone: phone_tx,
        simulate: phone.simulator(),
        nav: nav_tx,
    };
    tokio::spawn(run_vehicle(MockVehicle::new(), tx.clone()));
    tokio::spawn(run_media(MockMedia::new(), media_rx, tx.clone()));
    tokio::spawn(run_phone(phone, phone_rx, tx.clone()));
    tokio::spawn(run_gps(gps, pos_tx, tx.clone()));
    if std::env::var("NAV").is_ok_and(|v| v == "mock") {
        tokio::spawn(run_nav(MockNav, nav_rx, pos_rx, drive, tx.clone()));
    } else {
        tokio::spawn(run_nav(OsmNav, nav_rx, pos_rx, drive, tx.clone()));
    }
    tokio::spawn(run_can(MockCan::new(), tx.clone()));
    tokio::spawn(run_input(input, tx));

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .fallback_service(ServeDir::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../ui/dist")))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("http://127.0.0.1:3000  (stdin input: w/a/x/d, enter=select, q=back, +/-=knob)");
    axum::serve(listener, app).await.unwrap();
}

async fn run_vehicle(mut v: impl Vehicle, tx: Sender<String>) {
    let mut launch = LaunchTimer::new();
    loop {
        let s = v.next().await;
        let l = launch.update(Instant::now(), s.kmh);
        let _ = tx.send(Msg::VehicleSpeed { kmh: s.kmh }.into_json());
        let _ = tx.send(Msg::VehicleRpm { rpm: s.rpm }.into_json());
        let _ = tx.send(Msg::VehicleTelemetry(s.telemetry).into_json());
        let _ = tx.send(Msg::VehicleLaunch(l).into_json());
    }
}

async fn run_media(mut m: impl Media, mut rx: UnboundedReceiver<MediaCommand>, tx: Sender<String>) {
    loop {
        tokio::select! {
            Some(cmd) = rx.recv() => m.command(cmd).await,
            s = m.next() => {
                let _ = tx.send(Msg::MediaState(s).into_json());
            }
        }
    }
}

async fn run_phone(mut p: impl Phone, mut rx: UnboundedReceiver<PhoneCommand>, tx: Sender<String>) {
    loop {
        tokio::select! {
            Some(cmd) = rx.recv() => p.command(cmd).await,
            ev = p.next() => {
                let msg = match ev {
                    PhoneEvent::Notification(n) => Msg::PhoneNotification(n),
                    PhoneEvent::Call(c) => Msg::PhoneCall(c),
                };
                let _ = tx.send(msg.into_json());
            }
        }
    }
}

async fn run_gps(mut g: impl Gps, pos: watch::Sender<Option<(f64, f64)>>, tx: Sender<String>) {
    loop {
        let fix = g.next().await;
        pos.send_replace(fix.fix.then_some((fix.lat, fix.lon)));
        let _ = tx.send(Msg::GpsPosition(fix).into_json());
    }
}

async fn run_nav(
    mut nav: impl Nav,
    mut rx: UnboundedReceiver<Msg>,
    mut pos: watch::Receiver<Option<(f64, f64)>>,
    drive: UnboundedSender<Vec<(f64, f64)>>,
    tx: Sender<String>,
) {
    let send = |m: Msg| {
        let _ = tx.send(m.into_json());
    };
    let mut dests: Destinations =
        std::fs::read_to_string(DESTINATIONS_FILE).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
    let mut route: Option<Route> = None;
    let mut step = 0;
    loop {
        tokio::select! {
            Some(msg) = rx.recv() => match msg {
                Msg::NavSearch(query) => {
                    let places = if query.trim().is_empty() {
                        Ok(vec![])
                    } else {
                        let near = *pos.borrow();
                        nav.search(query.clone(), near).await
                    };
                    match places {
                        Ok(places) => send(Msg::NavResults { query, places }),
                        Err(e) => send(Msg::NavError(e)),
                    }
                }
                Msg::NavNavigate(place) => {
                    let from = *pos.borrow();
                    let Some(from) = from else {
                        send(Msg::NavError("Waiting for GPS fix".into()));
                        continue;
                    };
                    match nav.route(from, place.clone()).await {
                        Ok(r) => {
                            dests.recent.retain(|p| *p != place);
                            dests.recent.insert(0, place);
                            dests.recent.truncate(MAX_RECENT);
                            save(&dests);
                            send(Msg::NavDestinations(dests.clone()));
                            step = 1.min(r.steps.len().saturating_sub(1));
                            let _ = drive.send(r.geometry.clone());
                            send(Msg::NavRoute(Some(r.clone())));
                            send(Msg::NavProgress(progress(&r, &mut step, from)));
                            route = Some(r);
                        }
                        Err(e) => send(Msg::NavError(e)),
                    }
                }
                Msg::NavCancel => {
                    route = None;
                    send(Msg::NavRoute(None));
                }
                Msg::NavSave(place) => {
                    if dests.saved.contains(&place) {
                        dests.saved.retain(|p| *p != place);
                    } else {
                        dests.saved.push(place);
                    }
                    save(&dests);
                    send(Msg::NavDestinations(dests.clone()));
                }
                Msg::NavSync => {
                    send(Msg::NavDestinations(dests.clone()));
                    send(Msg::NavRoute(route.clone()));
                    let p = *pos.borrow();
                    if let (Some(r), Some(p)) = (&route, p) {
                        send(Msg::NavProgress(progress(r, &mut step, p)));
                    }
                }
                _ => {}
            },
            Ok(()) = pos.changed() => {
                let p = *pos.borrow_and_update();
                let (Some(r), Some(p)) = (&route, p) else { continue };
                let prog = progress(r, &mut step, p);
                if step + 1 >= r.steps.len() && prog.step_distance_m < REACHED_M {
                    send(Msg::NavArrived(r.destination.clone()));
                    send(Msg::NavRoute(None));
                    route = None;
                } else {
                    send(Msg::NavProgress(prog));
                }
            }
        }
    }
}

fn progress(r: &Route, step: &mut usize, p: (f64, f64)) -> Progress {
    let last = r.steps.len().saturating_sub(1);
    if let Some(i) = (*step..last).rev().find(|&i| distance_m(p, (r.steps[i].lat, r.steps[i].lon)) < REACHED_M) {
        *step = i + 1;
    }
    let next = &r.steps[*step];
    let step_distance_m = distance_m(p, (next.lat, next.lon));
    let remaining_m = step_distance_m + r.steps[*step..].iter().map(|s| s.distance_m).sum::<f64>();
    let remaining_s = if r.distance_m > 0.0 { r.duration_s * remaining_m / r.distance_m } else { 0.0 };
    Progress { step: *step, step_distance_m, remaining_m, remaining_s }
}

fn save(dests: &Destinations) {
    if let Err(e) = std::fs::write(DESTINATIONS_FILE, serde_json::to_string_pretty(dests).unwrap()) {
        eprintln!("saving destinations: {e}");
    }
}

async fn run_can(mut c: impl Can, tx: Sender<String>) {
    let mut frames = BTreeMap::<u32, (Vec<u8>, u64)>::new();
    let mut tick = tokio::time::interval(Duration::from_millis(200));
    loop {
        tokio::select! {
            f = c.next() => {
                let e = frames.entry(f.id).or_default();
                e.0 = f.data;
                e.1 += 1;
            }
            _ = tick.tick() => {
                let snapshot = frames.iter().map(|(&id, (data, count))| CanFrame { id, data: data.clone(), count: *count }).collect();
                let _ = tx.send(Msg::CanFrames(snapshot).into_json());
            }
        }
    }
}

async fn run_input(mut i: impl Input, tx: Sender<String>) {
    loop {
        let _ = tx.send(Msg::InputEvent(i.next().await).into_json());
    }
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| stream(socket, state))
}

async fn stream(mut socket: WebSocket, state: AppState) {
    let mut rx = state.tx.subscribe();
    loop {
        tokio::select! {
            msg = rx.recv() => match msg {
                Ok(text) => {
                    if socket.send(Message::Text(text.into())).await.is_err() {
                        break;
                    }
                }
                Err(RecvError::Lagged(_)) => {}
                Err(RecvError::Closed) => break,
            },
            msg = socket.recv() => match msg {
                Some(Ok(Message::Text(text))) => match Msg::from_json(&text) {
                    Ok(Msg::InputEvent(ev)) => {
                        let _ = state.input.send(ev);
                    }
                    Ok(Msg::MediaCommand(cmd)) => {
                        let _ = state.media.send(cmd);
                    }
                    Ok(Msg::PhoneCommand(cmd)) => {
                        let _ = state.phone.send(cmd);
                    }
                    Ok(Msg::PhoneSimulate(sim)) => {
                        let _ = state.simulate.send(sim);
                    }
                    Ok(m @ (Msg::NavSearch(_) | Msg::NavNavigate(_) | Msg::NavCancel | Msg::NavSave(_) | Msg::NavSync)) => {
                        let _ = state.nav.send(m);
                    }
                    Ok(_) => {}
                    Err(e) => eprintln!("bad message: {e}"),
                },
                Some(Ok(_)) => {}
                _ => break,
            },
        }
    }
}
