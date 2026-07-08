use crate::app::AppModel;
use relm4::RelmApp;
use std::sync::LazyLock;

mod action;
mod app;
mod app_navigation_ext;
mod app_render_ext;
mod chapter_parser;
mod components;
mod pages;
mod util;
mod workers;

// === THE FIX: Define the missing local Tokio runtime ===
pub static RUNTIME: LazyLock<tokio::runtime::Runtime> =
    LazyLock::new(|| tokio::runtime::Runtime::new().unwrap());

fn main() {
    podcasts_data::feed_manager::RUNTIME
        .set(&RUNTIME)
        .expect("Failed to share RUNTIME with feed manager.");

    gst::init().expect("Error initializing gstreamer");

    let app = RelmApp::new("org.flame.podcasts");
    app.run::<AppModel>(());
}
