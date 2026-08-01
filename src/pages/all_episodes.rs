use std::collections::BTreeMap;

use adw::prelude::*;
use chrono::Datelike;
use gst_play::PlayState;
use podcasts_data::{Episode, EpisodeId};
use relm4::{Component, prelude::*};

use crate::components::episode_group::{
    GroupedEpisodes, GroupedEpisodesInput, GroupedEpisodesOutput,
};

#[derive(Debug)]
pub struct AllEpisodesPage {
    groups: Vec<Controller<GroupedEpisodes>>,
}

#[derive(Debug)]
pub enum AllEpisodesPageInput {
    SetEpisodes(Vec<Episode>),
    DownloadStarted(EpisodeId),
    DownloadCancled(EpisodeId),
    DownloadProgress(EpisodeId, f64),
    DownloadFinished(EpisodeId),
    ChangePlayBackState(PlayState, EpisodeId),
    PlayBackProgress(EpisodeId, f64, u64),
    ChangeEpisodeTo(EpisodeId),
    EpisodeDeleted(EpisodeId),
}

#[derive(Debug)]
pub enum AllEpisodesPageOutput {
    TogglePlay(EpisodeId),
    RequestDownload(EpisodeId),
    CancleDownload(EpisodeId),
    SetPlayNext(EpisodeId),
    AddToPlaylist(EpisodeId),
    RequestDeleteEpisode(EpisodeId),
    NotifyError(String),
}

#[relm4::component(pub)]
impl Component for AllEpisodesPage {
    type Init = ();
    type Input = AllEpisodesPageInput;
    type Output = AllEpisodesPageOutput;
    type CommandOutput = ();

    view! {
        adw::NavigationPage {
            set_title: "Episodes",

           #[wrap(Some)]
            set_child = &adw::ToolbarView {

                 add_top_bar=&adw::HeaderBar {
                    set_show_title: false,
                 },

                #[wrap(Some)]
                set_content = &gtk::ScrolledWindow {
                    set_vexpand: true,
                    set_hscrollbar_policy: gtk::PolicyType::Never,

                    adw::Clamp {
                        set_maximum_size: 1100,
                        set_tightening_threshold: 900,

                        #[name="group_parent"]
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_margin_all: 12,
                            set_spacing: 6,
                        }
                    }

                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = AllEpisodesPage { groups: Vec::new() };
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
            AllEpisodesPageInput::SetEpisodes(mut episodes) => {
                episodes.sort_by(|a, b| b.epoch().cmp(&a.epoch()));
                let mut grouped: BTreeMap<i32, Vec<Episode>> = BTreeMap::new();

                for episode in episodes {
                    let year = episode.epoch().year();
                    grouped.entry(year).or_default().push(episode);
                }

                // Clear old groups (drops old controllers + their widgets should be removed too —
                // see note below about also clearing group_parent's children).
                self.groups.clear();

                for (year, episodes) in grouped.iter().rev() {
                    let group = GroupedEpisodes::builder()
                        .launch((format!("{}", *year), episodes.clone()))
                        .forward(sender.output_sender(), |msg| match msg {
                            GroupedEpisodesOutput::TogglePlay(episode_id) => {
                                AllEpisodesPageOutput::TogglePlay(episode_id)
                            }
                            GroupedEpisodesOutput::RequestDownload(episode_id) => {
                                AllEpisodesPageOutput::RequestDownload(episode_id)
                            }
                            GroupedEpisodesOutput::CancleDownload(episode_id) => {
                                AllEpisodesPageOutput::CancleDownload(episode_id)
                            }
                            GroupedEpisodesOutput::SetPlayNext(episode_id) => {
                                AllEpisodesPageOutput::SetPlayNext(episode_id)
                            }
                            GroupedEpisodesOutput::AddToPlaylist(episode_id) => {
                                AllEpisodesPageOutput::AddToPlaylist(episode_id)
                            }
                            GroupedEpisodesOutput::GotoEpisode(_episode_id) => todo!(),
                            GroupedEpisodesOutput::RequestDeleteEpisode(episode_id) => {
                                AllEpisodesPageOutput::RequestDeleteEpisode(episode_id)
                            }
                            GroupedEpisodesOutput::NotifyError(error) => {
                                AllEpisodesPageOutput::NotifyError(error)
                            }
                        });
                    widgets.group_parent.append(group.widget());
                    self.groups.push(group); // <-- keep it alive
                }
            }
            AllEpisodesPageInput::DownloadStarted(episode_id) => {
                for group in &self.groups {
                    group.emit(GroupedEpisodesInput::DownloadStarted(episode_id));
                }
            }
            AllEpisodesPageInput::DownloadCancled(episode_id) => {
                for group in &self.groups {
                    group.emit(GroupedEpisodesInput::DownloadCancled(episode_id));
                }
            }
            AllEpisodesPageInput::DownloadProgress(episode_id, fraction) => {
                for group in &self.groups {
                    group.emit(GroupedEpisodesInput::DownloadProgress(episode_id, fraction));
                }
            }
            AllEpisodesPageInput::DownloadFinished(episode_id) => {
                for group in &self.groups {
                    group.emit(GroupedEpisodesInput::DownloadFinished(episode_id));
                }
            }
            AllEpisodesPageInput::ChangePlayBackState(play_state, episode_id) => {
                for group in &self.groups {
                    group.emit(GroupedEpisodesInput::ChangePlayBackState(
                        play_state, episode_id,
                    ));
                }
            }
            AllEpisodesPageInput::PlayBackProgress(episode_id, pos, rem) => {
                for group in &self.groups {
                    group.emit(GroupedEpisodesInput::PlayBackProgress(episode_id, pos, rem));
                }
            }
            AllEpisodesPageInput::ChangeEpisodeTo(episode_id) => {
                for group in &self.groups {
                    group.emit(GroupedEpisodesInput::ChangeEpisodeTo(episode_id));
                }
            }
            AllEpisodesPageInput::EpisodeDeleted(episode_id) => {
                for group in &self.groups {
                    group.emit(GroupedEpisodesInput::EpisodeDeleted(episode_id));
                }
            }
        }
    }
}
