use adw::prelude::*;
use gst_play::PlayState;
use podcasts_data::{
    EpisodeId, EpisodeWidgetModel,
    dbqueries::{self, ShowFilter},
};
use relm4::{Component, prelude::*};

use crate::components::downloaded_episode_list_item::{
    DownloadedEpisodeListItem, DownloadedEpisodeListItemOutput,
};

#[derive(Debug)]
pub struct DownloadsPage {
    episodes: FactoryVecDeque<DownloadedEpisodeListItem>,
    is_loading: bool,
}

#[derive(Debug, Clone)]
pub enum DownloadsPageInput {
    FetchDownloads,
    GottenEpisodes(Vec<EpisodeWidgetModel>),
    DownloadStarted(EpisodeId),
    DownloadCancled(EpisodeId),
    DownloadProgress(EpisodeId, f64),
    DownloadFinished(EpisodeId),
    ChangePlayBackState(PlayState, EpisodeId),
    PlayBackProgress(EpisodeId, f64, u64),
    ChangeEpisodeTo(EpisodeId),
}

#[derive(Debug, Clone)]
pub enum DownloadsPageOutput {
    TogglePlay(EpisodeId),
    NotifyError(String),
    RequestDownload(EpisodeId),
    CancleDownload(EpisodeId),
}

#[relm4::component(pub)]
impl Component for DownloadsPage {
    type Init = ();
    type Input = DownloadsPageInput;
    type Output = DownloadsPageOutput;
    type CommandOutput = ();

    view! {
        adw::NavigationPage {
            set_title: "Podcast Details",

           #[wrap(Some)]
            set_child = &adw::ToolbarView {
                 add_top_bar=&adw::HeaderBar {
                    set_show_title: false,
                 },

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

                            #[local_ref]
                            episodes_container -> gtk::ListBox {
                                add_css_class: "boxed-list",
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
        let model = DownloadsPage {
            episodes: FactoryVecDeque::builder().launch(episodes_parent).forward(
                sender.output_sender(),
                |msg| match msg {
                    DownloadedEpisodeListItemOutput::TogglePlay(id) => {
                        DownloadsPageOutput::TogglePlay(id)
                    }
                    DownloadedEpisodeListItemOutput::RequestDownload(episode_id) => {
                        DownloadsPageOutput::RequestDownload(episode_id)
                    }
                    DownloadedEpisodeListItemOutput::CancleDownload(episode_id) => {
                        DownloadsPageOutput::CancleDownload(episode_id)
                    }
                    DownloadedEpisodeListItemOutput::NotifyError(error) => {
                        DownloadsPageOutput::NotifyError(error)
                    }
                },
            ),
            is_loading: true,
        };

        let episodes_container = model.episodes.widget();

        let widgets = view_output!();

        sender.input(DownloadsPageInput::FetchDownloads);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        match message {
            DownloadsPageInput::FetchDownloads => {
                let filter = ShowFilter {
                    any_downloaded: Some(true),
                    completed: None,
                    title_or_description: None,
                    reverse_order: true,
                };
                match dbqueries::get_podcasts_filter(&[], &filter) {
                    Ok(shows) => {
                        for show in shows.iter() {
                            let filter = dbqueries::EpisodeFilter {
                                downloaded: Some(true),
                                played: None,
                                search: None,
                                reverse_order: false,
                            };

                            match dbqueries::get_pd_episode_widgets(show, &filter) {
                                Ok(episodes) => {
                                    sender.input(DownloadsPageInput::GottenEpisodes(episodes));
                                }
                                Err(error) => {
                                    println!("Episode Error: {:?}", error);
                                }
                            }
                        }
                    }
                    Err(error) => {
                        println!("Error: {:?}", error);
                    }
                }
            }
            DownloadsPageInput::GottenEpisodes(episodes) => {
                let mut guard = self.episodes.guard();
                //guard.clear();

                for episode in episodes.iter() {
                    let index = guard.push_back(episode.clone());
                }
            }
            DownloadsPageInput::DownloadStarted(episode_id) => {}
            DownloadsPageInput::DownloadCancled(episode_id) => {}
            DownloadsPageInput::DownloadProgress(episode_id, _) => {}
            DownloadsPageInput::DownloadFinished(episode_id) => {}
            DownloadsPageInput::ChangePlayBackState(play_state, episode_id) => {}
            DownloadsPageInput::PlayBackProgress(episode_id, _, _) => {}
            DownloadsPageInput::ChangeEpisodeTo(episode_id) => {}
        }
    }
}
