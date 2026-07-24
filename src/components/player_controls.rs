use adw::prelude::*;
use gst_play::PlayState;
use podcasts_data::Episode;
use relm4::{Component, prelude::*};

use crate::components::progress_bar::{ProgressBar, ProgressBarInit, ProgressBarInput};

#[derive(Debug)]
pub struct PlayerControls {
    current_episode: Option<Episode>,
    image_texture: Option<adw::gdk::Texture>,
    current_state: PlayState,
    play_progress_bar: Controller<ProgressBar>,
}

#[derive(Debug)]
pub enum PlayerControlsInput {
    ChangePlayBackState(PlayState),
    SetCurrentEpisode(Episode),
    UpdateProgress(f64, u64),
    VolumeValue(f64),
    SetTexture(Option<adw::gdk::Texture>),
}

#[relm4::component(pub)]
impl Component for PlayerControls {
    type Init = ();
    type Input = PlayerControlsInput;
    type Output = ();
    type CommandOutput = ();

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let play_progress_bar = ProgressBar::builder()
            .launch(ProgressBarInit {
                initial_fraction: 0.0,
                interactive: true,
            })
            .detach();
        // .forward(sender.output_sender(), |msg| match msg {
        //     ProgressBarOutput::FractionChanged(fraction) => {
        //         MiniplayerModelOutput::SeekAudioPosition(fraction)
        //     }
        // });

        let model = PlayerControls {
            current_episode: None,
            image_texture: None,
            current_state: PlayState::Stopped,
            play_progress_bar,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            PlayerControlsInput::ChangePlayBackState(play_state) => {
                self.current_state = play_state;
            }
            PlayerControlsInput::SetCurrentEpisode(episode) => {
                self.current_episode = Some(episode);
            }
            PlayerControlsInput::UpdateProgress(pos, rem) => {
                self.play_progress_bar
                    .emit(ProgressBarInput::SetFraction(pos));
                let duration_str = if rem > 0 {
                    let hours = rem / 3600;
                    let minutes = (rem % 3600) / 60;
                    let seconds = rem % 60;

                    if hours > 0 {
                        format!("-{:02}:{:02}:{:02}", hours, minutes, seconds)
                    } else {
                        format!("-{:02}:{:02}", minutes, seconds)
                    }
                } else {
                    "-00:00".to_string()
                };
                widgets.remaining_time.set_label(&duration_str);
            }
            PlayerControlsInput::VolumeValue(vol) => {}
            PlayerControlsInput::SetTexture(texture) => {
                //self.image_texture = texture;

                match texture {
                    Some(tex) => {
                        let paintable = tex.upcast_ref::<adw::gdk::Paintable>();
                        widgets.cover_art.set_paintable(Some(paintable));
                        widgets.cover_art.set_visible(true);
                        widgets.cover_art.queue_resize();
                    }
                    None => {
                        widgets
                            .cover_art
                            .set_paintable(None::<&adw::gdk::Paintable>);
                        widgets.cover_art.set_visible(false);
                    }
                }
            }
        }
    }

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,
            set_spacing: 50,
            inline_css: "color: white;",

            gtk::Overlay {
                set_height_request: 350,
                set_width_request: 350,
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
                        set_label: &{
                            model.current_episode.as_ref()
                                .map(|s| s.title().trim().chars().take(2).collect::<String>().to_uppercase())
                                .unwrap_or_default()
                        },
                        add_css_class: "title-large",
                        set_hexpand: true,
                        set_vexpand: true,
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,
                        inline_css: "color: @dim_label_opacity; opacity: 0.25; font-weight: 800;",
                    }
                },

                #[name = "cover_art"]
                add_overlay = &gtk::Picture {
                    // #[watch]
                    // set_paintable: model.image_texture.as_ref().map(|t| t.upcast_ref::<adw::gdk::Paintable>()),
                    // #[watch]
                    // set_visible: model.image_texture.is_some(),
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
                set_halign: gtk::Align::Fill,
                set_hexpand: true,

                model.play_progress_bar.widget() {
                    set_height_request: 10,
                    set_halign: gtk::Align::Fill,
                    set_valign: gtk::Align::Center,
                },

                gtk::Box{
                    set_orientation: gtk::Orientation::Horizontal,
                    set_halign: gtk::Align::Fill,
                    set_hexpand: true,

                    gtk::Label {
                        set_label: "00:00",
                    },

                    gtk::Separator { set_hexpand: true, add_css_class: "spacer" },

                    #[name="remaining_time"]
                    gtk::Label {
                        set_label: "00:00",
                    },
                }

            },

            gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_halign: gtk::Align::Fill,
                    set_valign: gtk::Align::Center,
                    set_hexpand: true,

                    gtk::Separator { set_hexpand: true, add_css_class: "spacer" },

                    gtk::Button {
                        set_icon_name: "media-seek-backward-symbolic",
                        set_tooltip_text: Some("Rewind 15 seconds"),
                        set_valign: gtk::Align::Center,
                        set_css_classes: &["circular", "flat"],

                        #[watch]
                        set_sensitive: model.current_state != gst_play::PlayState::Stopped,

                        connect_clicked[sender]=>move |_| {
                           // let _  = sender.output(MiniplayerModelOutput::SeekBakward);
                        }
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
                               // let _ = sender.output(MiniplayerModelOutput::TogglePlay);
                            }
                        },
                    },

                    gtk::Button {
                        set_icon_name: "media-seek-forward-symbolic",
                        set_tooltip_text: Some("Fast forward 30 seconds"),
                        set_valign: gtk::Align::Center,
                        set_css_classes: &["circular", "flat"],

                        #[watch]
                        set_sensitive: model.current_state != gst_play::PlayState::Stopped,

                        connect_clicked[sender]=>move |_| {
                           // let _  = sender.output(MiniplayerModelOutput::Seekforward);
                        }
                    },

                    gtk::Separator { set_hexpand: true, add_css_class: "spacer" }

                }
        }
    }
}
