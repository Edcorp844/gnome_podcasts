use adw::prelude::*;
use gst_play::PlayState;
use podcasts_data::{Episode, EpisodeId, dbqueries};
use relm4::{
    Component, ComponentController, Controller, FactorySender, RelmWidgetExt,
    factory::{DynamicIndex, FactoryComponent},
};

use crate::{
    components::{
        circular_progress::{CircularProgress, CircularProgressInput},
        play_button::{
            self, EpisodePlayingState, PlayButton, PlayButtonInitData, PlayButtonInput,
            PlayButtonOutput,
        },
    },
    util::{
        cover_image::{ImageSize, fetch_cached_image},
        episode_description_parser,
    },
};

#[derive(Debug)]
pub struct EpisodeListItem {
    episode: Episode,
    texture: Option<adw::gdk::Texture>,
    play_button: Controller<PlayButton>,
    downloaded: bool,
    downloading: bool,
    progress_indicator: Controller<CircularProgress>,
}

#[derive(Debug, Clone)]
pub enum EpisodeListItemInput {
    ImageDownloaded(Option<adw::gdk::Texture>),
    TogglePlay,
    DownloadStarted,
    PlayBackProgress(f64, u64),
    DownloadProgress(f64),
    CancleDownload,
    DownloadCancled,
    RequestDownload,
    DownloadFinished,
    ChangePlayBackState(PlayState),
    ChangeEpisodeTo(EpisodeId),
}

#[derive(Debug)]
pub enum EpisodeListItemOutput {
    TogglePlay(EpisodeId),
    RequestDownload(EpisodeId),
    CancleDownload(EpisodeId),
    NotifyError(String),
}

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
                let downloaded_texture = fetch_cached_image(&image_url, ImageSize::default()).await;

                EpisodeListItemCmdInput::DownloadImage(downloaded_texture)
            });
        }

        let duration_str = match episode.duration() {
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

        let play_button = PlayButton::builder()
            .launch(PlayButtonInitData {
                label: duration_str,
                state: play_button::EpisodePlayingState::Stopped,
                progress: 0.0,
            })
            .forward(sender.input_sender(), |msg| match msg {
                PlayButtonOutput::Clicked => EpisodeListItemInput::TogglePlay,
            });

        let downloaded = {
            if let Ok(episode_widget) = dbqueries::get_episode_widget_from_id(episode.id()) {
                episode_widget.local_uri().is_some()
            } else {
                false
            }
        };

        let progress_indicator = CircularProgress::builder().launch(0.0).detach();

        Self {
            episode,
            texture: None,
            downloaded,
            downloading: false,
            progress_indicator,
            play_button,
        }
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            EpisodeListItemInput::ImageDownloaded(fetched_texture) => {
                self.texture = fetched_texture;
            }
            EpisodeListItemInput::TogglePlay => {
                let _ = sender.output(EpisodeListItemOutput::TogglePlay(self.episode.id()));
            }
            EpisodeListItemInput::CancleDownload => {
                let _ = sender.output(EpisodeListItemOutput::CancleDownload(self.episode.id()));
            }
            EpisodeListItemInput::DownloadCancled => todo!(),
            EpisodeListItemInput::RequestDownload => {
                let _ = sender.output(EpisodeListItemOutput::RequestDownload(self.episode.id()));
            }
            EpisodeListItemInput::DownloadStarted => {
                self.downloading = true;
            }
            EpisodeListItemInput::DownloadProgress(fraction) => {
                self.downloading = true;
                let _ = self
                    .progress_indicator
                    .sender()
                    .send(CircularProgressInput::SetFraction(fraction));
            }
            EpisodeListItemInput::DownloadFinished => {
                self.downloading = false;
                self.downloaded = true;
            }
            EpisodeListItemInput::ChangePlayBackState(state) => match state {
                PlayState::Stopped => {
                    self.play_button.emit(PlayButtonInput::UpdatePlayingState(
                        EpisodePlayingState::Stopped,
                    ));

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

                    self.play_button
                        .emit(PlayButtonInput::SetLabel(duration_str));
                }
                PlayState::Buffering => {
                    self.play_button.emit(PlayButtonInput::UpdatePlayingState(
                        EpisodePlayingState::Playing,
                    ));
                }
                PlayState::Paused => {
                    self.play_button.emit(PlayButtonInput::UpdatePlayingState(
                        EpisodePlayingState::Paused,
                    ));
                }
                PlayState::Playing => {
                    self.play_button.emit(PlayButtonInput::UpdatePlayingState(
                        EpisodePlayingState::Playing,
                    ));
                }
                _ => {}
            },
            EpisodeListItemInput::PlayBackProgress(fraction, rem) => {
                self.play_button
                    .emit(PlayButtonInput::UpdateProgress(fraction));
                let duration_str = if rem > 0 {
                    let hours = rem / 3600;
                    let minutes = (rem % 3600) / 60;
                    let seconds = rem % 60;

                    if hours > 0 {
                        format!("{}h {}m", hours, minutes)
                    } else if minutes > 0 {
                        format!("{}m", minutes)
                    } else {
                        format!("{}s", seconds)
                    }
                } else {
                    "0s".to_string()
                };

                self.play_button
                    .emit(PlayButtonInput::SetLabel(duration_str));
            }
            EpisodeListItemInput::ChangeEpisodeTo(episode_id) => {
                if episode_id != self.episode.id() {
                    sender.input(EpisodeListItemInput::ChangePlayBackState(
                        PlayState::Stopped,
                    ));
                }
            }
        }
    }

    fn update_cmd(&mut self, message: Self::CommandOutput, sender: FactorySender<Self>) {
        match message {
            EpisodeListItemCmdInput::DownloadImage(opt_texture) => {
                sender.input(EpisodeListItemInput::ImageDownloaded(opt_texture));
            }
        }
    }

    view! {
        gtk::Box {
            set_halign: gtk::Align::Fill,
            set_orientation: gtk::Orientation::Horizontal,
            set_margin_all: 16,

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_halign: gtk::Align::Start, // Locks the content layout tightly to the left
                set_spacing: 16,

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
                        box-shadow: 0 12px 28px rgba(0, 0, 0, 0.32);
                        border: 1px solid alpha(@borders, 0.8);
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
                        inline_css: "border-radius: 16px;", // Fixed missing radius value
                    }
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 8,
                    set_halign: gtk::Align::Start,
                    set_valign: gtk::Align::Start,

                    gtk::Label {
                        set_label: &self.episode.epoch().format("%e %b").to_string(),
                        add_css_class: "caption",
                        set_halign: gtk::Align::Start,
                        set_xalign: 0.0,
                        set_wrap: true
                    },

                    gtk::Label {
                        set_label: self.episode.title(),
                        add_css_class: "heading",
                        set_halign: gtk::Align::Start,
                        set_xalign: 0.0,
                        set_wrap: true
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
                        set_wrap: true,
                        set_lines: 3,
                        set_xalign: 0.0,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        set_css_classes: &vec!["dimmed", "body"]
                    },

                    gtk::Separator {
                        set_vexpand: true,
                        add_css_class: "spacer",
                        set_halign: gtk::Align::Start,
                    },

                    self.play_button.widget(),
                },
            },

            gtk::Box {
                set_hexpand: true,
            },

            gtk::Box {
                set_halign: gtk::Align::End,
                set_valign: gtk::Align::Center,

               gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,

                    gtk::Button {
                        #[watch]
                        set_visible: !self.downloading,
                        #[watch]
                        set_icon_name: if self.downloaded {
                            "object-select-symbolic"
                        } else {
                            "download-symbolic"
                        },
                        set_css_classes: &vec!["circular"],
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,

                        connect_clicked[sender] => move |_| {
                            sender.input(EpisodeListItemInput::RequestDownload);
                        }
                    },

                    gtk::Box {
                        #[watch]
                        set_visible: self.downloading,

                        self.progress_indicator.widget() {
                            set_size_request: (34, 34),
                            set_halign: gtk::Align::Center,
                            set_valign: gtk::Align::Center,
                        }
                    }
                }
            },
        }
    }
}
