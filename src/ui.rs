#![allow(non_snake_case)]
use crate::state;
use crate::sun;
use dioxus::prelude::*;
use serde::Deserialize;

use state::{clock_text, fetch_text, now_ms, open_url, platform, AppState, Size, Tile, COFFEE, LATEST_URL, VERSION};

static STATE: GlobalSignal<AppState> = Signal::global(AppState::load);
static NOW: GlobalSignal<i64> = Signal::global(now_ms);
static PAGE: GlobalSignal<&'static str> = Signal::global(|| "start");
static EDIT: GlobalSignal<bool> = Signal::global(|| false);
static NOTICE: GlobalSignal<String> = Signal::global(String::new);
static UPDATE_URL: GlobalSignal<String> = Signal::global(String::new);

const ACCENTS: [u32; 12] = [
    0x0078D7, 0x1FAEFF, 0x00ABA9, 0x00A300, 0x8CBF26, 0xFFC40D, 0xFF7F00, 0xE51400, 0xD80073,
    0xAA00FF, 0x6A00FF, 0x647687,
];

pub fn start() {
    dioxus::launch(App);
}

fn App() -> Element {
    use_effect(move || {
        spawn(async move {
            loop {
                sleep_ms(1000).await;
                *NOW.write() = now_ms();
            }
        });
    });
    use_effect(move || {
        STATE.read().save();
    });

    let dark = STATE.read().dark(*NOW.read());
    let aero = STATE.read().aero();
    let accent_n = STATE.read().accent;
    let accent = format!("#{:06X}", accent_n);
    let tint = format!("{}, {}, {}", (accent_n >> 16) & 255, (accent_n >> 8) & 255, accent_n & 255);
    let page = *PAGE.read();
    let mut class_name = String::from("app");
    if !dark {
        class_name.push_str(" light");
    }
    if aero {
        class_name.push_str(" aero");
    }

    rsx! {
        style { {include_str!("../assets/app.css")} }
        div {
            class: "{class_name}",
            style: "--accent: {accent}; --tint: {tint};",
            if page == "all" {
                AllApps {}
            } else if page == "settings" {
                Settings {}
            } else {
                Start {}
            }
            if !NOTICE.read().is_empty() {
                div { class: "notice",
                    span { "{NOTICE}" }
                    button {
                        onclick: move |_| {
                            let url = UPDATE_URL.read().clone();
                            if url.is_empty() {
                                *NOTICE.write() = String::new();
                            } else {
                                open_url(&url);
                            }
                        },
                        if UPDATE_URL.read().is_empty() { "Dismiss" } else { "Get update" }
                    }
                }
            }
        }
    }
}

fn Start() -> Element {
    let shape = STATE.read().shape.clone();
    let edit = *EDIT.read();
    let tiles = STATE.read().tiles.clone();
    let now = *NOW.read();
    rsx! {
        div { class: "stage",
            div { class: "grid",
                for tile in tiles {
                    TileFace { tile, shape: shape.clone(), edit, now }
                }
            }
        }
        div { class: "bar",
            button { onclick: move |_| { let on = *EDIT.read(); *EDIT.write() = !on; }, if edit { "DONE" } else { "EDIT" } }
            button { onclick: move |_| *PAGE.write() = "all", "ALL APPS" }
            button { onclick: move |_| *PAGE.write() = "settings", "⚙" }
        }
    }
}

#[component]
fn TileFace(tile: Tile, shape: String, edit: bool, now: i64) -> Element {
    let id = tile.id;
    let span = match tile.size {
        Size::Small => "tile s1",
        Size::Medium => "tile s2",
        Size::Wide => "tile s4",
    };
    let (time, date) = clock_text(now);
    let letter = tile.title.chars().next().unwrap_or('•').to_string();
    let href = tile.href.clone();
    let clock = tile.clock;
    let title = tile.title.clone();
    rsx! {
        div {
            class: "{span}",
            onclick: move |_| {
                if !*EDIT.read() && !clock {
                    open_url(&href);
                }
            },
            if clock {
                span { class: "clock-time", "{time}" }
                span { class: "clock-date", "{date}" }
            } else {
                span { class: "plate shape-{shape}", "{letter}" }
            }
            span { class: "label", "{title}" }
            if edit {
                span { class: "tools",
                    button { onclick: move |e| { e.stop_propagation(); resize(id); }, "{size_label(id)}" }
                    button { onclick: move |e| { e.stop_propagation(); nudge(id, -1); }, "←" }
                    button { onclick: move |e| { e.stop_propagation(); nudge(id, 1); }, "→" }
                    button { onclick: move |e| { e.stop_propagation(); unpin(id); }, "✕" }
                }
            }
        }
    }
}

fn AllApps() -> Element {
    let mut title = use_signal(String::new);
    let mut href = use_signal(String::new);
    let tiles = STATE.read().tiles.clone();
    rsx! {
        div { class: "panel",
            h1 { "All apps" }
            div { class: "list",
                for tile in tiles.iter().filter(|t| !t.clock) {
                    div { class: "item",
                        button {
                            onclick: {
                                let href = tile.href.clone();
                                move |_| open_url(&href)
                            },
                            "{tile.title}"
                        }
                    }
                }
            }
            h1 { "Pin a tile" }
            div { class: "row",
                input {
                    placeholder: "Name",
                    value: "{title}",
                    oninput: move |e| title.set(e.value()),
                }
                input {
                    placeholder: "https://",
                    value: "{href}",
                    oninput: move |e| href.set(e.value()),
                }
                button {
                    onclick: move |_| {
                        let name = title.read().trim().to_string();
                        if name.is_empty() { return; }
                        let link = href.read().trim().to_string();
                        let mut st = STATE.write();
                        let id = st.next_id;
                        st.next_id += 1;
                        st.tiles.push(Tile {
                            id,
                            title: name,
                            href: link,
                            size: Size::Small,
                            clock: false,
                        });
                        title.set(String::new());
                        href.set(String::new());
                    },
                    "Pin"
                }
            }
            div { class: "row",
                button { onclick: move |_| *PAGE.write() = "start", "Start" }
            }
        }
    }
}

fn Settings() -> Element {
    let theme = STATE.read().theme.clone();
    let look = STATE.read().look.clone();
    let shape = STATE.read().shape.clone();
    let accent = STATE.read().accent;
    let tiles = STATE.read().tiles.len();
    rsx! {
        div { class: "panel",
            h1 { "Settings" }
            p { class: "muted", "Look" }
            div { class: "row",
                look_btn { id: "metro", label: "Metro", current: look.clone() }
                look_btn { id: "aero", label: "Aero", current: look.clone() }
            }
            p { class: "muted", "Aero is see-through Vista glass on the tiles. Metro is the flat Start screen." }
            p { class: "muted", "Theme" }
            div { class: "row",
                theme_btn { id: "dark", label: "Dark", current: theme.clone() }
                theme_btn { id: "light", label: "Light", current: theme.clone() }
                theme_btn { id: "sun", label: "Auto", current: theme.clone() }
            }
            p { class: "muted", "Auto is light from sunrise to sunset, dark after." }
            p { class: "muted", "Accent" }
            div { class: "swatches",
                for color in ACCENTS {
                    button {
                        class: if color == accent { "swatch on" } else { "swatch" },
                        style: "background: #{color:06X};",
                        onclick: move |_| STATE.write().accent = color,
                    }
                }
            }
            p { class: "muted", "Icon shape" }
            div { class: "row",
                for id in ["system", "circle", "squircle", "rounded", "square", "teardrop"] {
                    button {
                        onclick: move |_| STATE.write().shape = id.to_string(),
                        if shape == id { "● {id}" } else { "{id}" }
                    }
                }
            }
            p { class: "muted", "Diagnostics  {VERSION}  ·  {platform()}  ·  {tiles} tiles" }
            div { class: "row",
                button { onclick: move |_| { spawn(check_update()); }, "Check for updates" }
                button { onclick: move |_| open_url(COFFEE), "Buy me a coffee" }
                button { onclick: move |_| *PAGE.write() = "start", "Start" }
            }
            p { class: "muted",
                "This build is the Start screen on every platform. On Android it can be chosen as the home app. iOS does not allow replacing the system home screen, so there it opens as an app. Updates install from the GitHub release for this platform."
            }
        }
    }
}

#[component]
fn theme_btn(id: &'static str, label: &'static str, current: String) -> Element {
    rsx! {
        button {
            onclick: move |_| STATE.write().theme = id.to_string(),
            if current == id { "● {label}" } else { "{label}" }
        }
    }
}

#[component]
fn look_btn(id: &'static str, label: &'static str, current: String) -> Element {
    rsx! {
        button {
            onclick: move |_| STATE.write().look = id.to_string(),
            if current == id { "● {label}" } else { "{label}" }
        }
    }
}

fn resize(id: u64) {
    if let Some(tile) = STATE.write().tiles.iter_mut().find(|t| t.id == id) {
        tile.size = tile.size.next();
    }
}

fn unpin(id: u64) {
    let mut st = STATE.write();
    if st.tiles.iter().any(|t| t.id == id && t.clock) {
        return;
    }
    st.tiles.retain(|t| t.id != id);
}

fn nudge(id: u64, dir: isize) {
    let mut st = STATE.write();
    let Some(i) = st.tiles.iter().position(|t| t.id == id) else { return };
    let j = i as isize + dir;
    if j < 0 || j >= st.tiles.len() as isize { return; }
    st.tiles.swap(i, j as usize);
}

fn size_label(id: u64) -> String {
    STATE.read().tiles.iter().find(|t| t.id == id).map(|t| t.size.label()).unwrap_or("").into()
}

#[derive(Deserialize)]
struct Latest {
    version: String,
    page: String,
}

async fn check_update() {
    let Some(raw) = fetch_text(LATEST_URL).await else {
        *NOTICE.write() = "Couldn't reach GitHub to check for an update.".into();
        *UPDATE_URL.write() = String::new();
        return;
    };
    let Ok(latest) = serde_json::from_str::<Latest>(&raw) else {
        *NOTICE.write() = "The update file was not readable.".into();
        *UPDATE_URL.write() = String::new();
        return;
    };
    if sun::is_newer(&latest.version, VERSION) {
        *NOTICE.write() = format!("Version {} is ready for {}.", latest.version, platform());
        *UPDATE_URL.write() = latest.page;
    } else {
        *NOTICE.write() = format!("You're on {VERSION}. That's the latest.");
        *UPDATE_URL.write() = String::new();
    }
}

async fn sleep_ms(ms: u64) {
    #[cfg(target_arch = "wasm32")]
    gloo_timers::future::TimeoutFuture::new(ms as u32).await;
    #[cfg(not(target_arch = "wasm32"))]
    {
        let (tx, rx) = futures_channel::oneshot::channel::<()>();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(ms));
            let _ = tx.send(());
        });
        let _ = rx.await;
    }
}
