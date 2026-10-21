mod state;
mod sun;

#[cfg(any(feature = "desktop", feature = "web", feature = "mobile"))]
mod ui;

#[cfg(any(feature = "desktop", feature = "web", feature = "mobile"))]
fn main() {
    ui::start();
}

#[cfg(not(any(feature = "desktop", feature = "web", feature = "mobile")))]
fn main() {}
