use crate::app::AppModel;
use gtk::prelude::*; // Required for StyleContext and Display traits
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

    let provider = gtk::CssProvider::new();

    let css_data = "
        @keyframes shimmer-flow {
            from { background-position: 0% 0%; }
            to { background-position: 200% 10%; }
        }
    ";

    provider.load_from_string(css_data);

    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    app.run::<AppModel>(());
}
