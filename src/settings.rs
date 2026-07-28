use gtk::gio::{
    Settings,
    prelude::{SettingsExt, SettingsExtManual},
};

pub trait AppSetting {
    fn schema_id() -> &'static str;

    /// Helper to get the GSettings object for this specific ID
    fn get_settings() -> Settings {
        Settings::new(Self::schema_id())
    }
}

pub struct GenaralSettings {
    settings: Settings,
}

impl AppSetting for GenaralSettings {
    fn schema_id() -> &'static str {
        "org.flame.podcasts"
    }
}

impl GenaralSettings {
    pub fn new() -> Self {
        Self {
            settings: Self::get_settings(),
        }
    }

    //-----SETTERS-----------
    pub fn set_libary_auto_sync(&self, value: bool) {
        let _ = self.settings.set_boolean("library-auto-sync", value);
    }

    pub fn set_search_platforms(&self, value: &[&str]) {
        let _ = self.settings.set_strv("search-platforms", value);
    }

    //----------GETTERS--------
     pub fn get_libary_auto_sync(&self)->bool {
      self.settings.boolean("library-auto-sync")
    }

    // pub fn set_search_platforms(&self, value: &[&str]) {
    //     let _ = self.settings.set_strv("search-platforms", value);
    // }
}
