use crate::{app::AppModel, components::app_menu::AppMenu};
use gettextrs::{LocaleCategory, bindtextdomain, setlocale, textdomain};
use relm4::{RelmApp, gtk::glib};
use std::sync::LazyLock;

#[macro_use]
extern crate log;

pub mod action;
pub mod app;
pub mod app_navigation_ext;
pub mod app_render_ext;
pub mod chapter_parser;
pub mod components;
pub mod config;
pub mod pages;
pub mod settings;
pub mod util;
pub mod workers;

#[cfg(test)]
fn init_gtk_tests() -> anyhow::Result<()> {
    gst::init()?;
    gtk::init()?;
    adw::init()?;
    Ok(())
}

pub static RUNTIME: LazyLock<tokio::runtime::Runtime> =
    LazyLock::new(|| tokio::runtime::Runtime::new().unwrap());
pub static MAINCONTEXT: LazyLock<glib::MainContext> = LazyLock::new(glib::MainContext::default);
pub static CHRONO_LOCALE: LazyLock<chrono::Locale> = LazyLock::new(|| {
    use std::str::FromStr;
    let system_locale = locale_config::Locale::current();
    let time_locale = system_locale.tags_for("time").next();
    let time_locale_str = time_locale.as_ref().map(|l| l.as_ref()).unwrap_or("C");
    let unix_formatted = time_locale_str.replace('-', "_");
    chrono::Locale::from_str(&unix_formatted).unwrap_or(chrono::Locale::POSIX)
});

fn main() {
    setlocale(LocaleCategory::LcAll, "");
    bindtextdomain(config::GETTEXT_PACKAGE, config::LOCALEDIR)
        .expect("Unable to bind the text domain");
    textdomain(config::GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    podcasts_data::feed_manager::RUNTIME
        .set(&RUNTIME)
        .expect("Failed to share RUNTIME with feed manager.");
    gst::init().expect("Error initializing gstreamer");
    AppMenu::register();
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
