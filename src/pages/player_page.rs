use adw::prelude::*;
use gst_play::PlayState;
use podcasts_data::{Episode, EpisodeId, dbqueries};
use relm4::{Component, prelude::*};

use crate::util::cover_image::{ImageSize, fetch_cached_image_with_gradient};

#[derive(Debug)]
pub struct PlayerPage {
    dynamic_gradient_css: String,
    current_episode: Option<Episode>,
    current_state: PlayState,
    texture: Option<adw::gdk::Texture>,
}

#[derive(Debug)]
pub enum PlayerPageInput {
    ImageDownloaded(Option<(adw::gdk::Texture, String)>),
    ChangePlayBackState(PlayState),
    SetCurrentEpisode(EpisodeId),
    UpdateProgress(f64, u64),
    VolumeValue(f64),
}

#[derive(Debug)]
pub enum PlayerPageOutput {
    ClosePlayer,
    NotifyError(String),
}

#[derive(Debug)]
pub enum PlayerPageCmdInput {
    DownloadImage(Option<(adw::gdk::Texture, String)>),
}

#[relm4::component(pub)]
impl Component for PlayerPage {
    type Init = ();
    type Input = PlayerPageInput;
    type Output = PlayerPageOutput;
    type CommandOutput = PlayerPageCmdInput;

    view! {
        adw::NavigationPage {
            // Set the title displayed in the navigation stack
            set_title: "Podcast Details",
             #[watch]
            inline_css: &model.dynamic_gradient_css,

           #[wrap(Some)]
            set_child = &adw::ToolbarView {

                 add_top_bar=&adw::HeaderBar {
                    set_show_title: false,
                    add_css_class: "flat",
                    inline_css: "background: transparent; box-shadow: none;",

                    pack_end=&gtk::Button {
                        set_tooltip_text: Some("Close"),
                        set_icon_name: "close-symbolic",

                        connect_clicked[sender]=> move |_| {
                            let _ = sender.output(PlayerPageOutput::ClosePlayer);
                        }
                    }
                 },

               #[wrap(Some)]
                set_content=&gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_hexpand: true,
                    set_vexpand: true,
                   
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = PlayerPage {
            texture: None,
            current_episode: None,
            current_state: PlayState::Stopped,
            dynamic_gradient_css: "background: rgba(0,0,0,1);".to_string(),
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
            PlayerPageInput::ImageDownloaded(pair) => {
                if let Some((texture, css)) = pair {
                    self.texture = Some(texture);
                    self.dynamic_gradient_css = css;
                }
            }
            PlayerPageInput::ChangePlayBackState(play_state) => {
                self.current_state = play_state;
            }
            PlayerPageInput::SetCurrentEpisode(episode_id) => {
                match dbqueries::get_episode_from_id(episode_id) {
                    Ok(episode) => {
                        let image_uri_opt = episode.image_uri().map(|s| s.to_string());
                        self.current_episode = Some(episode);

                        if let Some(image_uri) = image_uri_opt {
                            sender.oneshot_command(async move {
                                let downloaded_texture = fetch_cached_image_with_gradient(
                                    &image_uri,
                                    ImageSize::from_dimesion(500),
                                )
                                .await;

                                PlayerPageCmdInput::DownloadImage(downloaded_texture)
                            });
                        } else {
                            self.texture = None;
                        }
                    }
                    Err(error) => {
                        // Forward the database infrastructure errors up to the application logger
                        // let _ = sender.output(MiniplayerModelOutput::NotifyError(format!(
                        //     "Failed to resolve episode metadata: {:?}",
                        //     error
                        // )));
                    }
                }
            }
            PlayerPageInput::UpdateProgress(pos, rem) => {}
            PlayerPageInput::VolumeValue(val) => {}
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            PlayerPageCmdInput::DownloadImage(opt_texture) => {
                sender.input(PlayerPageInput::ImageDownloaded(opt_texture));
            }
        }
    }
}
