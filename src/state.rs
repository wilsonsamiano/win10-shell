use serde::{Deserialize, Serialize};

use crate::sun;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const LATEST_URL: &str =
    "https://raw.githubusercontent.com/wilsonsamiano/win10-shell/main/latest.json";
pub const COFFEE: &str = "https://buymeacoffee.com/wilsonsamiano";

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Size {
    Small,
    Medium,
    Wide,
}

impl Size {
    pub fn next(self) -> Self {
        match self {
            Size::Small => Size::Medium,
            Size::Medium => Size::Wide,
            Size::Wide => Size::Small,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Size::Small => "Small",
            Size::Medium => "Medium",
            Size::Wide => "Wide",
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Tile {
    pub id: u64,
    pub title: String,
    pub href: String,
    pub size: Size,
    pub clock: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AppState {
    pub tiles: Vec<Tile>,
    pub theme: String,
    pub accent: u32,
    pub shape: String,
    pub lat: f64,
    pub lon: f64,
    pub has_place: bool,
    pub next_id: u64,
}

impl Default for AppState {
    fn default() -> Self {
        let (lat, lon) = sun::timezone_place();
        Self {
            tiles: vec![
                Tile {
                    id: 1,
                    title: "Clock".into(),
                    href: String::new(),
                    size: Size::Wide,
                    clock: true,
                },
                Tile {
                    id: 2,
                    title: "GitHub".into(),
                    href: "https://github.com/wilsonsamiano/win10-shell".into(),
                    size: Size::Medium,
                    clock: false,
                },
                Tile {
                    id: 3,
                    title: "Coffee".into(),
                    href: COFFEE.into(),
                    size: Size::Small,
                    clock: false,
                },
            ],
            theme: "dark".into(),
            accent: 0x0078D7,
            shape: "rounded".into(),
            lat,
            lon,
            has_place: false,
            next_id: 4,
        }
    }
}

impl AppState {
    pub fn dark(&self, now_ms: i64) -> bool {
        match self.theme.as_str() {
            "light" => false,
            "sun" => {
                let (lat, lon) = if self.has_place {
                    (self.lat, self.lon)
                } else {
                    sun::timezone_place()
                };
                !sun::is_day(now_ms, lat, lon)
            }
            _ => true,
        }
    }

    pub fn load() -> Self {
        let raw = read_store();
        serde_json::from_str(&raw).unwrap_or_default()
    }

    pub fn save(&self) {
        if let Ok(raw) = serde_json::to_string(self) {
            write_store(&raw);
        }
    }
}

pub fn platform() -> &'static str {
    if cfg!(target_arch = "wasm32") {
        "pwa"
    } else if cfg!(target_os = "android") {
        "android"
    } else if cfg!(target_os = "ios") {
        "ios"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    }
}

pub fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

pub fn clock_text(now_ms: i64) -> (String, String) {
    use chrono::Datelike;
    use chrono::Timelike;
    let local = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(now_ms)
        .unwrap_or_else(chrono::Utc::now)
        .with_timezone(&chrono::Local);
    let (pm, hour) = local.hour12();
    let time = format!("{hour}:{:02}", local.minute());
    let wk = match local.weekday() {
        chrono::Weekday::Mon => "Mon",
        chrono::Weekday::Tue => "Tue",
        chrono::Weekday::Wed => "Wed",
        chrono::Weekday::Thu => "Thu",
        chrono::Weekday::Fri => "Fri",
        chrono::Weekday::Sat => "Sat",
        chrono::Weekday::Sun => "Sun",
    };
    let when = format!("{} · {wk} {}", if pm { "PM" } else { "AM" }, local.day());
    (time, when)
}

fn read_store() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|s| s.get_item("win10-shell").ok().flatten())
            .unwrap_or_default()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::fs::read_to_string(store_path()).unwrap_or_default()
    }
}

fn write_store(raw: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(s) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            let _ = s.set_item("win10-shell", raw);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = store_path();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(path, raw);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn store_path() -> std::path::PathBuf {
    dirs::data_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("win10-shell")
        .join("state.json")
}

pub fn open_url(url: &str) {
    if url.is_empty() {
        return;
    }
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(w) = web_sys::window() {
            let _ = w.open_with_url(url);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = open::that(url);
    }
}

pub async fn fetch_text(url: &str) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        gloo_net::http::Request::get(url)
            .send()
            .await
            .ok()?
            .text()
            .await
            .ok()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let url = url.to_string();
        let (tx, rx) = futures_channel::oneshot::channel();
        std::thread::spawn(move || {
            let body = ureq::get(&url)
                .timeout(std::time::Duration::from_secs(12))
                .call()
                .ok()
                .and_then(|r| r.into_string().ok());
            let _ = tx.send(body);
        });
        rx.await.ok().flatten()
    }
}
