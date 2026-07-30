use gettextrs::{LocaleCategory, bindtextdomain, setlocale, textdomain};
use relm4::RelmApp;
use xpodcasts::{RUNTIME, app::AppModel, components::app_menu::AppMenu, config};

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
