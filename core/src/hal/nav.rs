use crate::api::{Place, Route, Step};
use serde::Deserialize;
use std::future::Future;

pub trait Nav {
    fn search(&mut self, query: String, near: Option<(f64, f64)>) -> impl Future<Output = Result<Vec<Place>, String>> + Send;
    fn route(&mut self, from: (f64, f64), to: Place) -> impl Future<Output = Result<Route, String>> + Send;
}

const PHOTON: &str = "https://photon.komoot.io/api/";
const OSRM: &str = "https://router.project-osrm.org/route/v1/driving/";
const USER_AGENT: &str = "brz-headunit/0.1";
const RESULTS: &str = "8";

/// Photon (OSM geocoding) for search, OSRM for driving routes.
pub struct OsmNav;

impl Nav for OsmNav {
    async fn search(&mut self, query: String, near: Option<(f64, f64)>) -> Result<Vec<Place>, String> {
        let res: PhotonResponse = blocking(move || {
            let mut req = ureq::get(PHOTON).header("User-Agent", USER_AGENT).query("q", &query).query("limit", RESULTS);
            if let Some((lat, lon)) = near {
                req = req.query("lat", lat.to_string()).query("lon", lon.to_string());
            }
            req.call()?.body_mut().read_json()
        })
        .await?;
        Ok(res.features.into_iter().map(PhotonFeature::into_place).collect())
    }

    async fn route(&mut self, from: (f64, f64), to: Place) -> Result<Route, String> {
        let url = format!("{OSRM}{},{};{},{}", from.1, from.0, to.lon, to.lat);
        let res: OsrmResponse = blocking(move || {
            ureq::get(&url)
                .header("User-Agent", USER_AGENT)
                .query("overview", "full")
                .query("geometries", "geojson")
                .query("steps", "true")
                .call()?
                .body_mut()
                .read_json()
        })
        .await?;
        let r = res.routes.into_iter().next().ok_or_else(|| format!("No route found ({})", res.code))?;
        let steps = r.legs.into_iter().flat_map(|l| l.steps).map(OsrmStep::into_step).collect();
        Ok(Route {
            destination: to,
            distance_m: r.distance,
            duration_s: r.duration,
            geometry: r.geometry.coordinates.into_iter().map(|[lon, lat]| (lat, lon)).collect(),
            steps,
        })
    }
}

async fn blocking<T: Send + 'static>(f: impl FnOnce() -> Result<T, ureq::Error> + Send + 'static) -> Result<T, String> {
    match tokio::task::spawn_blocking(f).await {
        Ok(r) => r.map_err(|e| format!("Network error: {e}")),
        Err(e) => Err(e.to_string()),
    }
}

#[derive(Deserialize)]
struct PhotonResponse {
    features: Vec<PhotonFeature>,
}

#[derive(Deserialize)]
struct PhotonFeature {
    properties: PhotonProps,
    geometry: PointGeometry,
}

#[derive(Deserialize)]
struct PointGeometry {
    coordinates: [f64; 2],
}

#[derive(Deserialize)]
struct PhotonProps {
    name: Option<String>,
    housenumber: Option<String>,
    street: Option<String>,
    district: Option<String>,
    city: Option<String>,
    country: Option<String>,
}

impl PhotonFeature {
    fn into_place(self) -> Place {
        let p = self.properties;
        let street = match (p.housenumber, p.street) {
            (Some(n), Some(s)) => Some(format!("{n} {s}")),
            (_, s) => s,
        };
        let name = p.name.or(street.clone()).unwrap_or_else(|| "Dropped pin".into());
        let address = [street, p.district, p.city, p.country]
            .into_iter()
            .flatten()
            .filter(|s| *s != name)
            .collect::<Vec<_>>()
            .join(", ");
        let [lon, lat] = self.geometry.coordinates;
        Place { name, address, lat, lon }
    }
}

#[derive(Deserialize)]
struct OsrmResponse {
    code: String,
    #[serde(default)]
    routes: Vec<OsrmRoute>,
}

#[derive(Deserialize)]
struct OsrmRoute {
    distance: f64,
    duration: f64,
    geometry: LineGeometry,
    legs: Vec<OsrmLeg>,
}

#[derive(Deserialize)]
struct LineGeometry {
    coordinates: Vec<[f64; 2]>,
}

#[derive(Deserialize)]
struct OsrmLeg {
    steps: Vec<OsrmStep>,
}

#[derive(Deserialize)]
struct OsrmStep {
    distance: f64,
    name: String,
    maneuver: Maneuver,
}

#[derive(Deserialize)]
struct Maneuver {
    #[serde(rename = "type")]
    kind: String,
    modifier: Option<String>,
    exit: Option<u32>,
    location: [f64; 2],
}

impl OsrmStep {
    fn into_step(self) -> Step {
        let m = self.maneuver;
        let modifier = m.modifier.unwrap_or_else(|| "straight".into());
        let onto = if self.name.is_empty() { String::new() } else { format!(" onto {}", self.name) };
        let instruction = match m.kind.as_str() {
            "depart" if self.name.is_empty() => "Start driving".into(),
            "depart" => format!("Head along {}", self.name),
            "arrive" => "Arrive at destination".into(),
            "roundabout" | "rotary" => match m.exit {
                Some(n) => format!("At the roundabout, take exit {n}{onto}"),
                None => format!("Enter the roundabout{onto}"),
            },
            "exit roundabout" | "exit rotary" => format!("Exit the roundabout{onto}"),
            _ if modifier == "uturn" => format!("Make a U-turn{onto}"),
            kind => {
                let verb = match kind {
                    "fork" => "Keep",
                    "merge" => "Merge",
                    "on ramp" => "Take the ramp",
                    "off ramp" => "Take the exit",
                    "continue" | "new name" | "notification" => "Continue",
                    _ => "Turn",
                };
                if verb == "Turn" && modifier == "straight" {
                    format!("Continue straight{onto}")
                } else {
                    format!("{verb} {modifier}{onto}")
                }
            }
        };
        Step { instruction, modifier, distance_m: self.distance, lat: m.location[1], lon: m.location[0] }
    }
}

const PLACES: [(&str, &str, f64, f64); 8] = [
    ("Christchurch Airport", "Memorial Avenue, Christchurch", -43.4852, 172.5355),
    ("Riccarton Mall", "129 Riccarton Road, Christchurch", -43.5302, 172.5976),
    ("Christchurch Botanic Gardens", "Rolleston Avenue, Christchurch", -43.5310, 172.6205),
    ("Te Pae Christchurch", "100 Oxford Terrace, Christchurch", -43.5320, 172.6330),
    ("Sumner Beach", "The Esplanade, Sumner", -43.5683, 172.7575),
    ("Ruapuna Raceway", "Hasketts Road, Templeton", -43.5370, 172.4600),
    ("University of Canterbury", "Ilam Road, Christchurch", -43.5235, 172.5830),
    ("Lyttelton", "London Street, Lyttelton", -43.6033, 172.7194),
];

/// Offline: searches a fixed list of Christchurch places and routes in a straight line.
pub struct MockNav;

impl Nav for MockNav {
    async fn search(&mut self, query: String, _near: Option<(f64, f64)>) -> Result<Vec<Place>, String> {
        let q = query.to_lowercase();
        Ok(PLACES
            .iter()
            .filter(|(name, address, ..)| name.to_lowercase().contains(&q) || address.to_lowercase().contains(&q))
            .map(|&(name, address, lat, lon)| Place { name: name.into(), address: address.into(), lat, lon })
            .collect())
    }

    async fn route(&mut self, from: (f64, f64), to: Place) -> Result<Route, String> {
        let d = crate::hal::gps::distance_m(from, (to.lat, to.lon));
        let step = |instruction: &str, modifier: &str, distance_m, (lat, lon)| Step {
            instruction: instruction.into(),
            modifier: modifier.into(),
            distance_m,
            lat,
            lon,
        };
        Ok(Route {
            geometry: vec![from, (to.lat, to.lon)],
            steps: vec![
                step("Head towards destination", "straight", d, from),
                step("Arrive at destination", "straight", 0.0, (to.lat, to.lon)),
            ],
            distance_m: d,
            duration_s: d / (50.0 / 3.6),
            destination: to,
        })
    }
}
