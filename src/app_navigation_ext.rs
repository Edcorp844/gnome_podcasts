use gst_play::PlayState;
use podcasts_data::EpisodeId;
use relm4::{ComponentController, Controller};

use crate::pages::{
    home::{HomePage, HomePageInput},
    new::{NewPage, NewPageInput},
    podcast::{PodcastPage, PodcastPageInput},
    search::{SearchPage, SearchPageInput},
    shows::{ShowsPage, ShowsPageInput},
};

#[derive(Debug)]
pub enum NavigationPage {
    Search,
    Home,
    New,
    Shows,
    Library(String),
    Podcast,
}

impl NavigationPage {
    pub fn from_name(name: &str) -> Self {
        match name {
            "Search" => Self::Search,
            "Home" => Self::Home,
            "New" => Self::New,
            "Shows" => Self::Shows,
            "Podcast" => Self::Podcast,
            other => Self::Library(other.to_string()),
        }
    }

    pub fn to_key(&self) -> String {
        match self {
            Self::Search => "Search".to_string(),
            Self::Home => "Home".to_string(),
            Self::New => "New".to_string(),
            Self::Shows => "Shows".to_string(),
            Self::Podcast => "Podcast".to_string(),
            Self::Library(sub) => format!("Library_{}", sub),
        }
    }
}

#[derive(Debug)]

pub enum PageController {
    Search(Controller<SearchPage>),
    Home(Controller<HomePage>),
    New(Controller<NewPage>),
    Shows(Controller<ShowsPage>),
    Podcast(Controller<PodcastPage>),
    //Library(Controller<LibraryPage>),
}

impl PageController {
    pub(crate) fn widget(&self) -> &adw::NavigationPage {
        match self {
            Self::Search(c) => c.widget(),
            Self::Home(c) => c.widget(),
            Self::New(c) => c.widget(),
            Self::Shows(c) => c.widget(),
            Self::Podcast(c) => c.widget(),
        }
    }

    pub(crate) fn notify_download_finished(&self, episode_id: EpisodeId) {
        match self {
            Self::Search(c) => {
                c.emit(SearchPageInput::DownloadFinished(episode_id));
            }
            Self::Home(c) => {
                c.emit(HomePageInput::DownloadFinished(episode_id));
            }
            Self::New(c) => {
                c.emit(NewPageInput::DownloadFinished(episode_id));
            }
            Self::Shows(c) => {
                c.emit(ShowsPageInput::DownloadFinished(episode_id));
            }
            Self::Podcast(c) => {
                c.emit(PodcastPageInput::DownloadFinished(episode_id));
            }
        }
    }

    pub(crate) fn notify_download_started(&self, episode_id: EpisodeId) {
        match self {
            Self::Search(c) => {
                c.emit(SearchPageInput::DownloadStarted(episode_id));
            }
            Self::Home(c) => {
                c.emit(HomePageInput::DownloadStarted(episode_id));
            }
            Self::New(c) => {
                c.emit(NewPageInput::DownloadStarted(episode_id));
            }
            Self::Shows(c) => {
                c.emit(ShowsPageInput::DownloadStarted(episode_id));
            }
            Self::Podcast(c) => {
                c.emit(PodcastPageInput::DownloadStarted(episode_id));
            }
        }
    }

    pub(crate) fn notify_download_progress(&self, episode_id: EpisodeId, fraction: f64) {
        match self {
            Self::Search(c) => {
                c.emit(SearchPageInput::DownloadProgress(episode_id, fraction));
            }
            Self::Home(c) => {
                c.emit(HomePageInput::DownloadProgress(episode_id, fraction));
            }
            Self::New(c) => {
                c.emit(NewPageInput::DownloadProgress(episode_id, fraction));
            }
            Self::Shows(c) => {
                c.emit(ShowsPageInput::DownloadProgress(episode_id, fraction));
            }
            Self::Podcast(c) => {
                c.emit(PodcastPageInput::DownloadProgress(episode_id, fraction));
            }
        }
    }

    pub(crate) fn notify_playing_state(&self, episode_id: EpisodeId, state: PlayState) {
        match self {
            Self::Search(c) => {
                c.emit(SearchPageInput::ChangePlayBackState(state, episode_id));
            }
            Self::Home(c) => {
                c.emit(HomePageInput::ChangePlayBackState(state, episode_id));
            }
            Self::New(c) => {
                c.emit(NewPageInput::ChangePlayBackState(state, episode_id));
            }
            Self::Shows(c) => {
                c.emit(ShowsPageInput::ChangePlayBackState(state, episode_id));
            }
            Self::Podcast(c) => {
                c.emit(PodcastPageInput::ChangePlayBackState(state, episode_id));
            }
        }
    }

    pub(crate) fn notify_playback_progress(
        &self,
        episode_id: EpisodeId,
        fraction: f64,
        remaining_sec: u64,
    ) {
        match self {
            Self::Search(c) => {
                c.emit(SearchPageInput::PlayBackProgress(
                    episode_id,
                    fraction,
                    remaining_sec,
                ));
            }
            Self::Home(c) => {
                c.emit(HomePageInput::PlayBackProgress(
                    episode_id,
                    fraction,
                    remaining_sec,
                ));
            }
            Self::New(c) => {
                c.emit(NewPageInput::PlayBackProgress(
                    episode_id,
                    fraction,
                    remaining_sec,
                ));
            }
            Self::Shows(c) => {
                c.emit(ShowsPageInput::PlayBackProgress(
                    episode_id,
                    fraction,
                    remaining_sec,
                ));
            }
            Self::Podcast(c) => {
                c.emit(PodcastPageInput::PlayBackProgress(
                    episode_id,
                    fraction,
                    remaining_sec,
                ));
            }
        }
    }

    pub(crate) fn notify_current_episode(&self, episode_id: EpisodeId) {
        match self {
            Self::Search(c) => {
                c.emit(SearchPageInput::ChangeEpisodeTo(episode_id));
            }
            Self::Home(c) => {
                c.emit(HomePageInput::ChangeEpisodeTo(episode_id));
            }
            Self::New(c) => {
                c.emit(NewPageInput::ChangeEpisodeTo(episode_id));
            }
            Self::Shows(c) => {
                c.emit(ShowsPageInput::ChangeEpisodeTo(episode_id));
            }
            Self::Podcast(c) => {
                c.emit(PodcastPageInput::ChangeEpisodeTo(episode_id));
            }
        }
    }
}
