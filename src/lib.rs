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

pub static RUNTIME: std::sync::LazyLock<tokio::runtime::Runtime> =
    std::sync::LazyLock::new(|| tokio::runtime::Runtime::new().unwrap());
pub static MAINCONTEXT: std::sync::LazyLock<relm4::gtk::glib::MainContext> =
    std::sync::LazyLock::new(relm4::gtk::glib::MainContext::default);
pub static CHRONO_LOCALE: std::sync::LazyLock<chrono::Locale> = std::sync::LazyLock::new(|| {
    use std::str::FromStr;
    let system_locale = locale_config::Locale::current();
    let time_locale = system_locale.tags_for("time").next();
    let time_locale_str = time_locale.as_ref().map(|l| l.as_ref()).unwrap_or("C");
    let unix_formatted = time_locale_str.replace('-', "_");
    chrono::Locale::from_str(&unix_formatted).unwrap_or(chrono::Locale::POSIX)
});
