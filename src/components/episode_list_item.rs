use adw::prelude::*;
use gtk::gio::prelude::FileExt;
use podcasts_data::Episode;
use relm4::{
    FactorySender, RelmWidgetExt, factory::{DynamicIndex, FactoryComponent}
};

pub struct EpisodeListItem {
    episode: Episode,
    texture: Option<adw::gdk::Texture>,
}

#[derive(Debug)]
pub enum EpisodeListItemInput {
    ImageDownloaded(Option<adw::gdk::Texture>),
}

#[derive(Debug)]
pub enum EpisodeListItemOutput {}

#[derive(Debug)]
pub enum EpisodeListItemCmdInput {
    DownloadImage(Option<adw::gdk::Texture>),
}

#[relm4::factory(pub)]
impl FactoryComponent for EpisodeListItem {
    type Init = Episode;
    type Input = EpisodeListItemInput;
    type Output = EpisodeListItemOutput;
    type CommandOutput = EpisodeListItemCmdInput;
    type ParentWidget = gtk::ListBox;

    fn init_model(episode: Self::Init, _index: &DynamicIndex, sender: FactorySender<Self>) -> Self {
        let clone = episode.clone();

        if let Some(image_url_ref) = clone.image_uri() {
            let image_url = image_url_ref.to_string();

            sender.oneshot_command(async move {
                let texture_res = tokio::task::spawn_blocking(move || {
                    let load_image = || -> Option<gtk::gdk::Texture> {
                        let file = gtk::gio::File::for_uri(&image_url);
                        let (glib_bytes, _) = file.load_bytes(gtk::gio::Cancellable::NONE).ok()?;
                        gtk::gdk::Texture::from_bytes(&glib_bytes).ok()
                    };

                    load_image()
                })
                .await;

                let downloaded_texture = match texture_res {
                    Ok(Some(texture)) => Some(texture),
                    _ => None,
                };

                EpisodeListItemCmdInput::DownloadImage(downloaded_texture)
            });
        }

        Self {
            episode,
            texture: None,
        }
    }

    view! {
        gtk::Box{
            set_orientation: gtk::Orientation::Horizontal,
            set_margin_all: 16,
            set_halign: gtk::Align::Start,
            
            gtk::Overlay {
                                    set_height_request: 150,
                                    set_width_request: 150,
                                    set_halign: gtk::Align::Start,
                                    set_valign: gtk::Align::Start,

                                    #[wrap(Some)]
                                    set_child = &gtk::Box {
                                        set_orientation: gtk::Orientation::Vertical,
                                        set_hexpand: true,
                                        set_vexpand: true,
                                        set_halign: gtk::Align::Fill,
                                        set_valign: gtk::Align::Fill,

                                        inline_css: "
                                            background-color: mix(@window_bg_color, @card_fg_color, 0.1);
                                            border-radius: 16px;
                                            box-shadow: 0 12px 28px rgba(0, 0, 0, 0.15);
                                            border: 1px solid rgba(255, 255, 255, 0.05);
                                        ",

                                        gtk::Label {
                                            #[watch]
                                            set_label: &self.episode.title().trim().chars().take(2).collect::<String>().to_uppercase(),
                                            add_css_class: "title-large",
                                            set_hexpand: true,
                                            set_vexpand: true,
                                            set_halign: gtk::Align::Center,
                                            set_valign: gtk::Align::Center,
                                            inline_css: "color: @dim_label_opacity; opacity: 0.25; font-weight: 800;",
                                        }
                                    },

                                    add_overlay = &gtk::Picture {
                                        #[watch]
                                        set_paintable: self.texture.as_ref().map(|t| t.upcast_ref::<adw::gdk::Paintable>()),
                                        #[watch]
                                        set_visible: self.texture.is_some(),

                                        set_hexpand: true,
                                        set_vexpand: true,
                                        set_halign: gtk::Align::Fill,
                                        set_valign: gtk::Align::Fill,
                                        set_content_fit: gtk::ContentFit::Cover,
                                        set_can_shrink: true,
                                        inline_css: "border-radius: 16px; overflow: hidden;",
                                    }
                                },

            gtk::Box{
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 8,
                set_halign: gtk::Align::Start,
                set_valign: gtk::Align::Start,
                set_margin_start: 8,

                gtk::Label{
                    set_label:  &self.episode.epoch().format("%e %b").to_string(),
                    add_css_class: "caption",
                    set_halign: gtk::Align::Start,
                    set_wrap: true
                },

                gtk::Label{
                    set_label:  self.episode.title(),
                    add_css_class: "title-4",
                    set_halign: gtk::Align::Start,
                    set_wrap: true
                },

                gtk::Label{
                    set_label:  self.episode.description().unwrap_or(""),
                    add_css_class: "dimmed",
                    set_wrap: true
                },

                gtk::Separator{
                    set_vexpand: true,
                    add_css_class:"spacer"
                },

                gtk::Button{
                    set_label: &{
                        let duration_str = match self.episode.duration() {
                            Some(seconds) if seconds > 0 => {
                                let hours = seconds / 3600;
                                let minutes = (seconds % 3600) / 60;

                                if hours > 0 {
                                    format!("{}h {}m", hours, minutes)
                                } else {
                                    format!("{}m", minutes)
                                }
                            }
                            _ => "0m".to_string(),
                        };
                        format!("▶ {}", duration_str)
                    },
                    set_css_classes: &vec!["pill"],
                    set_halign: gtk::Align::Start,
                    set_valign: gtk::Align::Start

                }
            }
        }
    }
}
