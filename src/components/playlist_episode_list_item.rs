use adw::prelude::*;
use podcasts_data::{Episode, dbqueries};
use relm4::factory::FactoryComponent;
use relm4::{FactorySender, RelmWidgetExt};

use crate::util::cover_image::{ImageSize, fetch_cached_image};
use crate::util::episode_description_parser;

#[derive(Debug)]
pub struct PlayListEpisodeListItem {
    episode: Episode,
    texture: Option<adw::gdk::Texture>,
}

#[derive(Debug)]
pub enum PlayListEpisodeListItemInput {
    ImageDownloaded(Option<adw::gdk::Texture>),
}

#[derive(Debug)]
pub enum PlayListEpisodeListItemOutput {
    NotifyError(String),
}

#[derive(Debug)]
pub enum PlayListEpisodeListItemCmdInput {
    DownloadImage(Option<adw::gdk::Texture>),
}

#[relm4::factory(pub)]
impl FactoryComponent for PlayListEpisodeListItem {
    type Init = Episode;
    type Input = PlayListEpisodeListItemInput;
    type Output = PlayListEpisodeListItemOutput;
    type CommandOutput = PlayListEpisodeListItemCmdInput;
    type ParentWidget = gtk::Box;

    view! {
         gtk::Box {
            set_halign: gtk::Align::Fill,
            set_orientation: gtk::Orientation::Horizontal,
            set_margin_all: 16,

           gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_halign: gtk::Align::Start, // Locks the content layout to the left
                set_spacing: 16,

                gtk::Overlay {
                    set_height_request: 80,
                    set_width_request: 80,
                    set_halign: gtk::Align::Start,
                    set_valign: gtk::Align::Start,

                    #[wrap(Some)]
                    set_child = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_hexpand: true,
                        set_vexpand: true,
                        add_css_class: "frame",
                        set_halign: gtk::Align::Fill,
                        set_valign: gtk::Align::Fill,
                        inline_css: "background-color: mix(var(--window-bg-color), var(--card-fg-color), 0.1); border-radius: 16px; box-shadow: 0 12px 28px rgba(0, 0, 0, 0.32); ",

                        gtk::Label {
                            #[watch]
                            set_label: &self.episode.title().trim().chars().take(2).collect::<String>().to_uppercase(),
                            set_css_classes: &vec!["title-large", "dimmed"],
                            set_hexpand: true,
                            set_vexpand: true,
                            set_halign: gtk::Align::Center,
                            set_valign: gtk::Align::Center,
                            inline_css: "opacity: 0.25; font-weight: 800;",
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
                        inline_css: "border-radius: 16px;",
                    }
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 8,
                    set_halign: gtk::Align::Start,
                    set_valign: gtk::Align::Start,

                    gtk::Label {
                        set_label: self.episode.title(),
                        //add_css_class: "title-4",
                        set_halign: gtk::Align::Start,
                        set_xalign: 0.0,
                        set_wrap: false
                    },

                    gtk::Label {
                        #[watch]
                        set_use_markup: true,
                        #[watch]
                        set_markup: &{
                            let raw_markup = if let Some(desc) = self.episode.description() {
                                let markup = episode_description_parser::html2pango_markup(desc);

                                if markup.is_empty() && !desc.is_empty() {
                                    html2text::config::plain()
                                        .string_from_read(desc.as_bytes(), desc.len())
                                        .unwrap_or_else(|_| desc.to_string())
                                } else {
                                    markup
                                }
                            } else {
                                "".to_string()
                            };
                            raw_markup.replace('\n', " ").replace('\r', " ")
                        },
                        set_halign: gtk::Align::Start,
                        set_wrap: false,
                        set_lines: 1,
                        set_xalign: 0.0,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        set_css_classes: &vec!["dimmed", "body"]
                    },

                },

           },

           gtk::Box {
                set_hexpand: true,
            },

            gtk::Box {
                set_halign: gtk::Align::End,
                set_valign: gtk::Align::Center,

                gtk::MenuButton {
                    set_icon_name: "view-more-symbolic",
                    set_css_classes: &vec!["circular", "flat"],
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,

                    #[wrap(Some)]
                    #[name="popover"]
                    set_popover = &gtk::Popover {
                        set_autohide: true,

                        #[wrap(Some)]
                        set_child = &gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,

                            gtk::Button {
                                set_css_classes: &vec!["flat"],
                                #[wrap(Some)]
                                set_child = &adw::ButtonContent {
                                    set_icon_name: "display-projector-symbolic",
                                    set_label: "Go to show",
                                    set_halign: gtk::Align::Start,
                                },
                            },

                            gtk::Button {
                                set_css_classes: &vec!["flat"],
                                #[wrap(Some)]
                                set_child = &adw::ButtonContent {
                                    set_icon_name: "user-trash-symbolic",
                                    set_label: "Delete Episode",
                                    set_halign: gtk::Align::Start,
                                },
                                connect_clicked[sender, popover] => move |_| {
                                    popover.popdown();
                                    //sender.input(DownloadedEpisodeListItemInput::RequestDelete);
                                },
                            },
                        },
                    },
                },
            }
        },

    }

    fn init_model(
        episode: Self::Init,
        index: &Self::Index,
        sender: relm4::prelude::FactorySender<Self>,
    ) -> Self {
        match dbqueries::get_episode_from_id(episode.id()) {
            Ok(ep) => {
                if let Some(image_url_ref) = ep.image_uri() {
                    let image_url = image_url_ref.to_string();

                    sender.oneshot_command(async move {
                        let downloaded_texture =
                            fetch_cached_image(&image_url, ImageSize::default()).await;

                        PlayListEpisodeListItemCmdInput::DownloadImage(downloaded_texture)
                    });
                }
            }
            Err(e) => {
                let _ = sender.output(PlayListEpisodeListItemOutput::NotifyError(e.to_string()));
            }
        }
        Self {
            episode,
            texture: None,
        }
    }

    fn update(&mut self, message: Self::Input, sender: relm4::prelude::FactorySender<Self>) {
        match message {
            PlayListEpisodeListItemInput::ImageDownloaded(texture) => {
                self.texture = texture;
            }
        }
    }

    fn update_cmd(&mut self, message: Self::CommandOutput, sender: FactorySender<Self>) {
        match message {
            PlayListEpisodeListItemCmdInput::DownloadImage(opt_texture) => {
                sender.input(PlayListEpisodeListItemInput::ImageDownloaded(opt_texture));
            }
        }
    }
}
