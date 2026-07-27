use adw::prelude::*;
use gst_play::PlayState;
use podcasts_data::{EpisodeId, dbqueries};
use relm4::{Component, prelude::*};

use crate::{
    components::player_controls::{PlayerControls, PlayerControlsInput, PlayerControlsOutput},
    util::{
        cover_image::{ImageSize, fetch_cached_image},
        gradient_extractor::GradientColorExtractor,
    },
};

#[derive(Debug)]
pub struct PlayerPage {
    player_controls: Controller<PlayerControls>,
}

#[derive(Debug)]
pub enum PlayerPageInput {
    ImageDownloaded(Option<adw::gdk::Texture>),
    ChangePlayBackState(PlayState),
    SetCurrentEpisode(EpisodeId),
    UpdateProgress(f64, u64),
    VolumeValue(f64),
}

#[derive(Debug)]
pub enum PlayerPageOutput {
    NotifyError(String),
    TogglePlay,
    SeekAudioPosition(f64),
    Seekforward,
    SeekBakward,
}

#[derive(Debug)]
pub enum PlayerPageCmdInput {
    DownloadImage(Option<adw::gdk::Texture>),
}

#[relm4::component(pub)]
impl Component for PlayerPage {
    type Init = ();
    type Input = PlayerPageInput;
    type Output = PlayerPageOutput;
    type CommandOutput = PlayerPageCmdInput;

    view! {
        #[name="page"]
        adw::NavigationPage {
            inline_css: "background: rgba(0,0,0,1);",

            #[wrap(Some)]
            set_child = &adw::ToolbarView {

                 add_top_bar=&adw::HeaderBar {
                    set_show_title: false,
                    add_css_class: "flat",
                    inline_css: "background: transparent; box-shadow: none;",
                 },

               #[wrap(Some)]
                set_content=&gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_hexpand: true,
                    set_vexpand: true,



                    model.player_controls.widget(){

                    }

                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let player_controls =
            PlayerControls::builder()
                .launch(())
                .forward(sender.output_sender(), |msg| match msg {
                    PlayerControlsOutput::TogglePlay => PlayerPageOutput::TogglePlay,
                    PlayerControlsOutput::SeekAudioPosition(pos) => {
                        PlayerPageOutput::SeekAudioPosition(pos)
                    }
                    PlayerControlsOutput::Seekforward => PlayerPageOutput::Seekforward,
                    PlayerControlsOutput::SeekBakward => PlayerPageOutput::SeekBakward,
                });
        let model = PlayerPage { player_controls };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            PlayerPageInput::ImageDownloaded(texture) => match texture {
                Some(texture) => {
                    self.player_controls
                        .emit(PlayerControlsInput::SetTexture(Some(texture.clone())));

                    widgets.page.inline_css(
                        &GradientColorExtractor::extract_css_gradient_from_texture(&texture),
                    );
                }
                None => {
                    let _ = sender.output(PlayerPageOutput::NotifyError(format!(
                        "Player Error: Failed to Load Image texture",
                    )));
                }
            },
            PlayerPageInput::ChangePlayBackState(play_state) => {
                self.player_controls
                    .emit(PlayerControlsInput::ChangePlayBackState(play_state));
            }
            PlayerPageInput::SetCurrentEpisode(episode_id) => {
                match dbqueries::get_episode_from_id(episode_id) {
                    Ok(episode) => {
                        let image_uri_opt = episode.image_uri().map(|s| s.to_string());

                        self.player_controls
                            .emit(PlayerControlsInput::SetCurrentEpisode(episode));

                        if let Some(image_uri) = image_uri_opt {
                            sender.oneshot_command(async move {
                                let downloaded_texture =
                                    fetch_cached_image(&image_uri, ImageSize::from_dimesion(450))
                                        .await;

                                PlayerPageCmdInput::DownloadImage(downloaded_texture)
                            });
                        } else {
                        }
                    }
                    Err(error) => {
                        let _ = sender.output(PlayerPageOutput::NotifyError(format!(
                            "Failed to resolve episode metadata: {:?}",
                            error
                        )));
                    }
                }
            }
            PlayerPageInput::UpdateProgress(pos, rem) => {
                self.player_controls
                    .emit(PlayerControlsInput::UpdateProgress(pos, rem));
            }
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
