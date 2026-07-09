use adw::prelude::*;
use gst::format;
use gst_play::PlayState;
use podcasts_data::Episode;
use podcasts_data::EpisodeId;
use podcasts_data::EpisodeModel;
use podcasts_data::dbqueries;
use relm4::prelude::*;

use crate::components::volume_scale::VolumeControlInit;
use crate::components::volume_scale::VolumeControlModel;
use crate::components::volume_scale::VolumeControlOutput;
use crate::util::cover_image::ImageSize;
use crate::util::cover_image::fetch_cached_image;

pub struct MiniPlayerModel {
    pub volume_slider: Controller<VolumeControlModel>,
    texture: Option<adw::gdk::Texture>,
    current_state: PlayState,
    current_episode: Option<Episode>,
}

#[derive(Debug)]
pub enum MiniplayerModelInput {
    HandleVolumeChange(f64),
    ImageDownloaded(Option<adw::gdk::Texture>),
    ChangePlayBackState(PlayState),
    SetCurrentEpisode(EpisodeId),
}

#[derive(Debug)]
pub enum MiniplayerModelOutput {
    TogglePlay,
    NotifyError(String),
}

#[derive(Debug)]
pub enum MiniPlayerModelCmdInput {
    DownloadImage(Option<adw::gdk::Texture>),
}

#[relm4::component(pub)]
impl Component for MiniPlayerModel {
    type Init = ();
    type Input = MiniplayerModelInput;
    type Output = MiniplayerModelOutput;
    type CommandOutput = MiniPlayerModelCmdInput;

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let volume_slider = VolumeControlModel::builder()
            .launch(VolumeControlInit {
                initial_volume: 0.5,
            })
            .forward(sender.input_sender(), |output| match output {
                VolumeControlOutput::VolumeChanged(new_vol) => {
                    MiniplayerModelInput::HandleVolumeChange(new_vol)
                }
            });

        let model = Self {
            volume_slider,
            texture: None,
            current_state: PlayState::Stopped,
            current_episode: None,
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            MiniplayerModelInput::ImageDownloaded(fetched_texture) => {
                self.texture = fetched_texture;
            }
            MiniplayerModelInput::ChangePlayBackState(state) => {
                self.current_state = state;
            }
            MiniplayerModelInput::SetCurrentEpisode(id) => {
                match dbqueries::get_episode_from_id(id) {
                    Ok(episode) => {
                        let image_uri_opt = episode.image_uri().map(|s| s.to_string());
                        self.current_episode = Some(episode);

                        if let Some(image_uri) = image_uri_opt {
                            sender.oneshot_command(async move {
                                let downloaded_texture =
                                    fetch_cached_image(&image_uri, ImageSize::from_dimesion(50))
                                        .await;

                                MiniPlayerModelCmdInput::DownloadImage(downloaded_texture)
                            });
                        } else {
                            self.texture = None;
                        }
                    }
                    Err(error) => {
                        // Forward the database infrastructure errors up to the application logger
                        let _ = sender.output(MiniplayerModelOutput::NotifyError(format!(
                            "Failed to resolve episode metadata: {:?}",
                            error
                        )));
                    }
                }
            }
            _ => {}
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            MiniPlayerModelCmdInput::DownloadImage(opt_texture) => {
                sender.input(MiniplayerModelInput::ImageDownloaded(opt_texture));
            }
        }
    }

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            inline_css: "
                background: @secondary_sidebar_bg_color; 
                box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05), 0 4px 12px rgba(0, 0, 0, 0.04);
                border-bottom: 1px solid rgba(0, 0, 0, 0.03);
            ",

            adw::HeaderBar {
                set_valign: gtk::Align::Start,

                // --- Media Playback Controls ---
                pack_start = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,

                    gtk::Separator { set_hexpand: true, add_css_class: "spacer" },

                    gtk::Button {
                        set_icon_name: "media-seek-backward-symbolic",
                        set_tooltip_text: Some("Rewind 10 seconds"),
                        set_valign: gtk::Align::Center,
                        set_css_classes: &["circular", "flat"],

                        #[watch]
                        set_sensitive: model.current_state != gst_play::PlayState::Stopped,
                    },

                     match model.current_state {
                        gst_play::PlayState::Buffering => adw::Spinner{
                            set_size_request: (24, 24),
                            set_margin_start: 6,
                            set_margin_end: 6,
                            set_halign: gtk::Align::Center,
                            set_valign: gtk::Align::Center,
                        }
                        _=> gtk::Button {
                            set_tooltip_text: Some("Play"),
                            add_css_class: "circular",
                            set_size_request: (50, 50),
                            set_margin_start: 6,
                            set_margin_end: 6,

                            #[watch]
                            set_sensitive: model.current_state != gst_play::PlayState::Stopped,

                            #[wrap(Some)]
                            set_child = &gtk::Image {
                                #[watch]
                                set_icon_name: if model.current_state == gst_play::PlayState::Playing {
                                    Some("media-playback-pause-symbolic")
                                } else {
                                    Some("media-playback-start-symbolic")
                                },
                                set_icon_size: gtk::IconSize::Large,
                            },

                            connect_clicked[sender] => move |_| {
                                let _ = sender.output(MiniplayerModelOutput::TogglePlay);
                            }
                        },
                    },

                    gtk::Button {
                        set_icon_name: "media-seek-forward-symbolic",
                        set_tooltip_text: Some("Fast forward 10 seconds"),
                        set_valign: gtk::Align::Center,
                        set_css_classes: &["circular", "flat"],

                        #[watch]
                        set_sensitive: model.current_state != gst_play::PlayState::Stopped,
                    },



                    gtk::Separator { set_hexpand: true, add_css_class: "spacer" },
                },


                #[wrap(Some)]
                set_title_widget =&gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 8,
                    set_halign: gtk::Align::Fill,
                    set_hexpand: true,
                    set_vexpand: false,

                    gtk::Separator { set_hexpand: true, add_css_class: "spacer" },

                    gtk::Box {
                        set_width_request: 50,
                        set_height_request: 50,
                        set_valign: gtk::Align::Center,
                        set_halign: gtk::Align::Center,
                        set_hexpand: false,
                        set_vexpand: false,
                        add_css_class: "suggested-action",
                        inline_css: "
                            background: rgba(255, 255, 255, 0.7);
                            border-radius: 8px;
                            min-width: 50px;
                            max-width: 50px;
                            min-height: 50px;
                            max-height: 50px;
                        ",

                        gtk::Picture {
                            #[watch]
                            set_paintable: model.texture.as_ref().map(|t| t.upcast_ref::<adw::gdk::Paintable>()),
                            #[watch]
                            set_visible: model.texture.is_some(),
                            set_hexpand: false,
                            set_vexpand: false,
                            set_halign: gtk::Align::Fill,
                            set_valign: gtk::Align::Fill,
                            set_content_fit: gtk::ContentFit::Cover,
                            set_can_shrink: true,

                            inline_css: "
                                border-radius: 8px;
                                overflow: hidden;
                                min-width: 50px;
                                max-width: 50px;
                                min-height: 50px;
                                max-height: 50px;
                            ",
                        }
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_halign: gtk::Align::Start,
                        set_spacing: 4,


                        gtk::Label {
                            #[watch]
                            set_label:match &model.current_episode {
                                Some(episode) => episode.title(),
                                None => "No Track Selected",
                            },
                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                            set_lines: 1,
                            set_halign: gtk::Align::Start,
                            set_valign: gtk::Align::Center,
                        },

                        gtk::Label {
                            set_label: "July 5",
                            set_halign: gtk::Align::Start,
                            set_valign: gtk::Align::Center,
                            add_css_class: "dimmed",
                        }
                    },

                    gtk::Separator { set_hexpand: true, add_css_class: "spacer" },
                },

                // --- Volume Slider on the Right End ---
                pack_end = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_valign: gtk::Align::Center,

                    gtk::Separator { set_hexpand: true, add_css_class: "spacer" },

                    // Appends the custom volume slider widget controller cleanly
                    append = model.volume_slider.widget(),

                    gtk::Separator { set_hexpand: true, add_css_class: "spacer" },
                }
            }
        },
    }
}
