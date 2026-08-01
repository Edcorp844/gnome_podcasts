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

        @keyframes eq-bounce {
            0%   { transform: scaleY(0.3); }
            50%  { transform: scaleY(1.0); }
            100% { transform: scaleY(0.3); }
        }

        .eq-bar {
            background-color: white;
            min-width: 4px;
            min-height: 24px;
            border-radius: 2px;
            transform-origin: bottom;
            animation: eq-bounce 0.9s ease-in-out infinite;
        }

        .eq-bar-1 { animation-delay: 0s; }
        .eq-bar-2 { animation-delay: 0.15s; }
        .eq-bar-3 { animation-delay: 0.3s; }
        .eq-bar-4 { animation-delay: 0.45s; }
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
