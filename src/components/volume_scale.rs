use gtk::prelude::*;
use relm4::prelude::*;

const VOL_STEP: f64 = 0.05;
const VOL_LOWER: f64 = 0.0;
const VOL_UPPER: f64 = 1.0;

pub struct VolumeControlInit {
    pub initial_volume: f64,
}

pub struct VolumeControlModel {
    volume: f64,
    prev_volume: f64,
    is_muted: bool,
}

#[derive(Debug)]
pub enum VolumeControlInput {
    ScrollVolume(f64),
    SliderChanged(f64),
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
        let clamped_vol = init.initial_volume.clamp(VOL_LOWER, VOL_UPPER);
        let model = VolumeControlModel {
            volume: clamped_vol,
            prev_volume: if clamped_vol > 0.0 { clamped_vol } else { 1.0 },
            is_muted: clamped_vol == 0.0,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            VolumeControlInput::ScrollVolume(dy) => {
                let delta = dy * VOL_STEP;
                let next_vol = (self.volume - delta).clamp(VOL_LOWER, VOL_UPPER);
                
                if (self.volume - next_vol).abs() > f64::EPSILON {
                    self.volume = next_vol;
                    self.is_muted = self.volume == VOL_LOWER;
                    if self.volume > 0.0 {
                        self.prev_volume = self.volume;
                    }
                    let _ = sender.output(VolumeControlOutput::VolumeChanged(self.volume));
                }
            }
            VolumeControlInput::SliderChanged(new_vol) => {
                if (self.volume - new_vol).abs() > f64::EPSILON {
                    self.volume = new_vol;
                    self.is_muted = self.volume == VOL_LOWER;
                    if self.volume > 0.0 {
                        self.prev_volume = self.volume;
                    }
                    let _ = sender.output(VolumeControlOutput::VolumeChanged(self.volume));
                }
            }
            VolumeControlInput::ToggleMute => {
                if self.is_muted {
                    self.volume = self.prev_volume;
                    self.is_muted = false;
                } else {
                    self.prev_volume = self.volume;
                    self.volume = 0.0;
                    self.is_muted = true;
                }
                let _ = sender.output(VolumeControlOutput::VolumeChanged(self.volume));
            }
        }
    }

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 8,
            set_width_request: 200,
            
            // The template group layout properties
            set_accessible_role: gtk::AccessibleRole::Group,
           

            // Lower / Mute Button
            append = &gtk::Button {
                set_has_frame: false,
                set_valign: gtk::Align::Center,
                set_icon_name: if model.is_muted {
                    "audio-volume-muted-symbolic"
                } else {
                    "audio-volume-low-symbolic"
                },
                
                connect_clicked => VolumeControlInput::ToggleMute,
            },

            // The main pill-shaped accent slider scale element
            append = &gtk::Scale {
                set_orientation: gtk::Orientation::Horizontal,
                set_hexpand: true,
                
                // Sizing and alignment attributes mapping the image parameters
                set_size_request: (-1, 24),
                set_margin_vertical: 6,
               

                // Accent design applied completely inline without subnodes strings
                // inline_css: "
                //     background-color: @accent_color;
                //     border-radius: 12px;
                //     box-shadow: none;
                //     border: none;
                // ",

                set_adjustment: &gtk::Adjustment::new(
                    model.volume,
                    VOL_LOWER,
                    VOL_UPPER,
                    VOL_STEP,
                    0.0,
                    0.0
                ),
                set_value: model.volume,

                connect_value_changed[sender] => move |scale| {
                    sender.input(VolumeControlInput::SliderChanged(scale.value()));
                },

                add_controller = gtk::EventControllerScroll {
                    set_name: Some("volume-scroll"),
                    set_flags: gtk::EventControllerScrollFlags::VERTICAL,

                    connect_scroll[sender] => move |_, _, dy| {
                        sender.input(VolumeControlInput::ScrollVolume(dy));
                        gtk::glib::Propagation::Stop
                    }
                }
            },
        }
    }
}