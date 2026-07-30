use std::collections::HashMap;

use adw::prelude::*;
use gst_play::PlayState;
use podcasts_data::{Episode, EpisodeId, dbqueries};
use relm4::{Component, prelude::*};

use crate::components::episode_list_item::{
    EpisodeListItem, EpisodeListItemInput, EpisodeListItemOutput,
};

#[derive(Debug)]
pub struct RecentlyUpdatedPage {
    episodes: FactoryVecDeque<EpisodeListItem>,
    index_by_id: HashMap<EpisodeId, relm4::factory::DynamicIndex>,
    is_loading: bool,
}

#[derive(Debug, Clone)]
pub enum RecentlyUpdatedPageInput {
    FetchDownloads,
    GottenEpisodes(Vec<Episode>),
    DownloadStarted(EpisodeId),
    DownloadCancled(EpisodeId),
    DownloadProgress(EpisodeId, f64),
    DownloadFinished(EpisodeId),
    ChangePlayBackState(PlayState, EpisodeId),
    PlayBackProgress(EpisodeId, f64, u64),
    ChangeEpisodeTo(EpisodeId),
    EpisodeDeleted(EpisodeId),
}

#[derive(Debug, Clone)]
pub enum RecentlyUpdatedPageOutput {
    TogglePlay(EpisodeId),
    NotifyError(String),
    RequestDownload(EpisodeId),
    CancleDownload(EpisodeId),
    StartLoading,
    StopLoading,
}

#[relm4::component(pub)]
impl Component for RecentlyUpdatedPage {
    type Init = ();
    type Input = RecentlyUpdatedPageInput;
    type Output = RecentlyUpdatedPageOutput;
    type CommandOutput = ();

    view! {
        adw::NavigationPage {
            set_title: "Downloads Page",

           #[wrap(Some)]
            set_child = &adw::ToolbarView {

               #[wrap(Some)]
                set_content= &gtk::ScrolledWindow {
                    set_vexpand: true,
                    set_hscrollbar_policy: gtk::PolicyType::Never,

                    adw::Clamp {
                        set_maximum_size: 1100,
                        set_tightening_threshold: 900,

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_margin_all: 12,
                            set_spacing: 6,

                             gtk::Label {
                                set_margin_top: 40,
                                set_margin_horizontal: 20,
                                set_label: "Recently Updated",
                                set_halign:gtk::Align::Start,

                                add_css_class: "title-1"
                            },

                            #[local_ref]
                            episodes_container -> gtk::ListBox {
                                #[watch]
                                set_visible: !model.episodes.is_empty(),
                                set_margin_all: 20,
                                add_css_class: "boxed-list",
                            },

                           adw::StatusPage {
                                #[watch]
                                set_visible: model.episodes.is_empty(),

                                set_title: "You recently updated podcast episodes will appear here",
                                set_icon_name: Some("media-optical-symbolic"),

                                set_vexpand: true,
                                set_hexpand: true,
                            },
                        }
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
        let episodes_parent = gtk::ListBox::builder().build();
        let model = RecentlyUpdatedPage {
            episodes: FactoryVecDeque::builder().launch(episodes_parent).forward(
                sender.output_sender(),
                |msg| match msg {
                    EpisodeListItemOutput::TogglePlay(id) => {
                        RecentlyUpdatedPageOutput::TogglePlay(id)
                    }
                    EpisodeListItemOutput::NotifyError(error) => {
                        RecentlyUpdatedPageOutput::NotifyError(error)
                    }
                    EpisodeListItemOutput::RequestDownload(episode_id) => {
                        RecentlyUpdatedPageOutput::RequestDownload(episode_id)
                    }
                    EpisodeListItemOutput::CancleDownload(episode_id) => {
                        RecentlyUpdatedPageOutput::CancleDownload(episode_id)
                    }
                },
            ),
            is_loading: true,
            index_by_id: HashMap::new(),
        };

        let episodes_container = model.episodes.widget();

        let widgets = view_output!();

        sender.input(RecentlyUpdatedPageInput::FetchDownloads);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            RecentlyUpdatedPageInput::FetchDownloads => {
                self.is_loading = true;
                let _ = sender.output(RecentlyUpdatedPageOutput::StartLoading);

                match dbqueries::get_episodes() {
                    Ok(episodes) => {
                        let new_episodes = episodes.into_iter().take(50).collect();

                        sender.input(RecentlyUpdatedPageInput::GottenEpisodes(new_episodes));
                    }
                    Err(error) => {
                        let _ = sender
                            .output(RecentlyUpdatedPageOutput::NotifyError(error.to_string()));
                    }
                }

                self.is_loading = false;
                let _ = sender.output(RecentlyUpdatedPageOutput::StopLoading);
            }
            RecentlyUpdatedPageInput::GottenEpisodes(episodes) => {
                let mut guard = self.episodes.guard();
                //guard.clear();

                for episode in episodes.iter() {
                    let index = guard.push_back(episode.clone());
                    self.index_by_id.insert(episode.id(), index);
                }
            }
            RecentlyUpdatedPageInput::DownloadStarted(episode_id) => {
                dbg!(episode_id);
            }
            RecentlyUpdatedPageInput::DownloadCancled(episode_id) => {}
            RecentlyUpdatedPageInput::DownloadProgress(episode_id, _) => {}
            RecentlyUpdatedPageInput::DownloadFinished(episode_id) => {}
            RecentlyUpdatedPageInput::ChangePlayBackState(play_state, episode_id) => {
                if let Some(index) = self.index_by_id.get(&episode_id) {
                    self.episodes.send(
                        index.current_index(),
                        EpisodeListItemInput::ChangePlayBackState(play_state),
                    );
                }
            }
            RecentlyUpdatedPageInput::PlayBackProgress(episode_id, pos, rem) => {
                if let Some(index) = self.index_by_id.get(&episode_id) {
                    self.episodes.send(
                        index.current_index(),
                        EpisodeListItemInput::PlayBackProgress(pos, rem),
                    );
                }
            }
            RecentlyUpdatedPageInput::ChangeEpisodeTo(episode_id) => {
                self.episodes
                    .broadcast(EpisodeListItemInput::ChangeEpisodeTo(episode_id));
            }
            RecentlyUpdatedPageInput::EpisodeDeleted(episode_id) => {
                if let Some(index) = self.index_by_id.get(&episode_id) {
                    let mut guard = self.episodes.guard();
                    guard.remove(index.current_index());
                }
            }
        }
    }
}
