use adw::prelude::*;
use relm4::prelude::*;

use crate::components::volume_scale::VolumeControlInit;
use crate::components::volume_scale::VolumeControlModel;
use crate::components::volume_scale::VolumeControlOutput;



pub struct MiniPlayerModel {
    pub volume_slider: Controller<VolumeControlModel>,
    texture: Option<adw::gdk::Texture>,
}

#[derive(Debug)]
pub enum MiniplayerModelInput {
    HandleVolumeChange(f64),
    ImageDownloaded(Option<adw::gdk::Texture>),
}

#[derive(Debug)]
pub enum MiniPlayerModelCmdInput {
    DownloadImage(Option<adw::gdk::Texture>),
}

#[relm4::component(pub)]
impl Component for MiniPlayerModel {
    type Init = ();
    type Input = MiniplayerModelInput;
    type Output = ();
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

        let url_string = "https://is1-ssl.mzstatic.com/image/thumb/Podcasts125/v4/3e/8d/84/3e8d8499-7b3b-5be9-e808-e7e533904632/mza_12520670224895083138.jpg/540x540bb.webp";

        sender.oneshot_command(async move {
            let texture_res = tokio::task::spawn_blocking(move || {
                let load_image = || -> Option<gtk::gdk::Texture> {
                    // 1. No .ok()? needed here because for_uri returns a raw File directly
                    let file = gtk::gio::File::for_uri(&url_string);

                    // 2. load_bytes still returns a Result, so keep .ok()? here
                    let (glib_bytes, _) = file.load_bytes(gtk::gio::Cancellable::NONE).ok()?;

                    const THUMB_SIZE: i32 = 50;
                    let stream = gtk::gio::MemoryInputStream::from_bytes(&glib_bytes);
                    let pixbuf = gtk::gdk_pixbuf::Pixbuf::from_stream_at_scale(
                        &stream,
                        THUMB_SIZE,
                        THUMB_SIZE,
                        true, // preserve aspect ratio (source is already square, so this lands on exactly 50x50)
                        gtk::gio::Cancellable::NONE,
                    )
                    .ok()?;

                    Some(gtk::gdk::Texture::for_pixbuf(&pixbuf))
                };

                load_image()
            })
            .await;

            let downloaded_texture = match texture_res {
                Ok(Some(texture)) => Some(texture),
                _ => None,
            };

            MiniPlayerModelCmdInput::DownloadImage(downloaded_texture)
        });

        let model = Self {
            volume_slider,
            texture: None,
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            MiniplayerModelInput::ImageDownloaded(fetched_texture) => {
                self.texture = fetched_texture;
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
                    },

                    gtk::Button {
                        set_tooltip_text: Some("Play"),
                        add_css_class: "circular",
                        set_size_request: (50, 50),
                        set_margin_start: 6,
                        set_margin_end: 6,

                        #[wrap(Some)]
                        set_child = &gtk::Image {
                            set_icon_name: Some("media-playback-start-symbolic"),
                            set_icon_size: gtk::IconSize::Large,
                        }
                    },

                    gtk::Button {
                        set_icon_name: "media-seek-forward-symbolic",
                        set_tooltip_text: Some("Fast forward 10 seconds"),
                        set_valign: gtk::Align::Center,
                        set_css_classes: &["circular", "flat"],
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
                        // These are floors, not caps — the CSS min/max below
                        // is what actually locks the size regardless of the
                        // source image's real (540x540) dimensions.
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

                            // No hexpand/vexpand here — we don't want this
                            // widget asking the HeaderBar for more room.
                            // ContentFit::Cover still crops-to-fill within
                            // whatever fixed box it's given below.
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
                            set_label: "S.393 why Many Struggle To Believe",
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