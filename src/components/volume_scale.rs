use gtk::prelude::*;
use relm4::prelude::*;

use crate::components::progress_bar::{ProgressBar, ProgressBarInit};

pub struct VolumeControlInit {
    pub initial_volume: f64,
}

pub struct VolumeControlModel {
    volume_bar: Controller<ProgressBar>,
    prev_volume: f64,
    is_muted: bool,
}

#[derive(Debug)]
pub enum VolumeControlInput {
    SetVolume(f64),
    ToggleMute,
}

#[derive(Debug)]
pub enum VolumeControlOutput {
    VolumeChanged(f64),
}

#[relm4::component(pub)]
impl Component for VolumeControlModel {
    type Init = VolumeControlInit;
    type Input = VolumeControlInput;
    type Output = VolumeControlOutput;
    type CommandOutput = ();

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let volume_bar = ProgressBar::builder()
            .launch(ProgressBarInit {
                initial_fraction: 0.5,
                interactive: true,
            })
            .detach();

        let model = VolumeControlModel {
            volume_bar,
            prev_volume: 0.5,
            is_muted: false,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 8,

            gtk::MenuButton {
                set_icon_name: "audio-volume-low-symbolic",

                 #[wrap(Some)]
                set_popover = &gtk::Popover {
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 8,
                        set_margin_all: 4,

                        gtk::Image{
                            set_icon_name: Some("audio-volume-low-symbolic"),
                        },

                        model.volume_bar.widget(){
                            set_size_request: (200, 10),
                            set_halign: gtk::Align::Center,
                            set_valign: gtk::Align::Center,
                        },

                        gtk::Image{
                            set_icon_name: Some("audio-volume-high-symbolic"),
                        },
                    }
                }
            }


        }
    }
}

