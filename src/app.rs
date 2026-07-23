use crate::action::Action;
use crate::app_navigation_ext::NavigationPage;
use crate::app_navigation_ext::PageController;
use crate::components::main_menu_button::MainMenuButton;
use crate::components::miniplayer::MiniPlayerModel;

use crate::components::miniplayer::MiniplayerModelInput;
use crate::components::miniplayer::MiniplayerModelOutput;
use crate::components::miniplayer::PlayerPageView;
use crate::pages::downloads::DownloadsPage;
use crate::pages::home::HomPageOutPut;
use crate::pages::home::HomePage;
use crate::pages::new::NewPage;
use crate::pages::player_page::PlayerPage;
use crate::pages::player_page::PlayerPageInput;
use crate::pages::player_page::PlayerPageOutput;
use crate::pages::search::SearchPage;
use crate::pages::search::SearchPageOutput;
use crate::pages::shows::ShowsPage;
use crate::pages::shows::ShowsPageInput;
use crate::pages::shows::ShowsPageOutput;
use crate::workers::action_worker::service::ActionWorker;
use crate::workers::action_worker::service::ActionWorkerInput;
use crate::workers::action_worker::service::ActionWorkerOutput;
use gst_play::PlayState;
use podcasts_data::EpisodeId;
use relm4::ComponentParts;
use relm4::adw::prelude::*;
use relm4::prelude::*;
use std::collections::HashMap;

pub struct AppModel {
    is_sidebar_visible: bool,
    main_menu_button: Controller<MainMenuButton>,
    player_page: Controller<PlayerPage>,
    pages_cache: HashMap<String, PageController>,
    current_page_key: String,
    miniplayer: Controller<MiniPlayerModel>,
    worker_controller: Controller<ActionWorker>,
    is_loading: bool,
    show_full_player: bool,
}

#[derive(Debug)]
pub enum AppModelInput {
    ClosePlayer,
    ToggleSidebar,
    StartLoading,
    StopLoading,
    ShowSearchPage,
    TogglePlayBack,
    SelectPage(NavigationPage),
    SetSidebarCollapsed(bool),
    RefreshShowsPage,
    SeekAudioPosition(f64),
    TogglePlay(EpisodeId),
    NotifyError(String),
    Subscribe(String),
    ChangePlayBackState(PlayState, EpisodeId),
    PlayBackProgress(EpisodeId, f64, u64),
    SetCurrentEpisode(EpisodeId),
    RequestDownload(EpisodeId),
    CancleDownload(EpisodeId),
    DownloadStarted(EpisodeId),
    DownloadCancled(EpisodeId),
    DownloadProgress(EpisodeId, f64),
    DownloadFinished(EpisodeId),
    RequestDeleteEpisode(EpisodeId),
    EpisodeDeleted(EpisodeId),
    SetVolume(f64),
    VolumeValue(f64),
    ShowPlayerPage(PlayerPageView),
    RequestMute,
    RequestUnmute,
    RequestVolumeValue,
    Seekforward,
    SeekBakward,
    None,
}

#[derive(Debug)]
pub enum AppModelOutput {}

#[relm4::component(pub)]
impl Component for AppModel {
    type Init = ();
    type Input = AppModelInput;
    type Output = AppModelOutput;
    type CommandOutput = ();

    view! {
        adw::ApplicationWindow {
            set_default_size: (1080 ,800),

            #[wrap(Some)]
            #[name = "nav_view"]
            set_content = &gtk::Stack {
                // Watch the model state to automatically flip between pages
                #[watch]
                set_visible_child_name: if model.show_full_player { "player_page" } else { "main_view" },

                // PAGE 1: Your original split-view application layout
                add_named[Some("main_view")] = &adw::OverlaySplitView {
                    #[watch]
                    set_show_sidebar: model.is_sidebar_visible,

                    set_pin_sidebar: true,
                    set_sidebar_position: gtk::PackType::Start,
                    set_min_sidebar_width: 260.0,
                    set_max_sidebar_width: 320.0,

                    connect_collapsed_notify => move |view| {
                        AppModelInput::SetSidebarCollapsed(view.is_collapsed());
                    },

                    #[wrap(Some)]
                    set_sidebar = &adw::ToolbarView {
                        add_top_bar = &adw::HeaderBar {
                            #[name(search_button)]
                            pack_start = &gtk::ToggleButton {
                                set_icon_name: "edit-find-symbolic",
                                set_tooltip: "Search",
                                add_css_class: "flat",

                                connect_clicked[sender] => move |btn| {
                                    if btn.is_active() {
                                        sender.input(AppModelInput::ShowSearchPage);
                                    }
                                }
                            },
                            pack_end = model.main_menu_button.widget(),
                        },

                        #[wrap(Some)]
                        set_content = &gtk::ScrolledWindow {
                            set_hscrollbar_policy: gtk::PolicyType::Never,

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 8,

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 8,

                                    #[name = "pages"]
                                    gtk::ListBox {
                                        set_selection_mode: gtk::SelectionMode::None,
                                        set_margin_start: 12,
                                        set_margin_end: 12,
                                        add_css_class: "navigation-sidebar",
                                    }
                                },

                                #[name = "library_header"]
                                gtk::Box {
                                    set_margin_start: 16,
                                    set_margin_end: 16,
                                    set_margin_horizontal: 32,

                                    gtk::Label {
                                        set_label: "Library",
                                        set_halign: gtk::Align::Start,
                                        add_css_class: "dim-label"
                                    },
                                    gtk::Separator { set_hexpand: true , add_css_class: "spacer"},
                                    #[name = "library_chevron"]
                                    gtk::Image { set_icon_name: Some("pan-down-symbolic"), add_css_class: "dim-label" }
                                },

                                #[name = "library_revealer"]
                                gtk::Revealer {
                                    set_reveal_child: true,

                                    #[name = "library"]
                                    gtk::ListBox {
                                        set_selection_mode: gtk::SelectionMode::None,
                                        set_margin_start: 12,
                                        set_margin_end: 12,
                                        add_css_class: "navigation-sidebar",
                                    }
                                },
                            },
                        }
                    },

                    #[wrap(Some)]
                    #[name = "toast_overlay"]
                    set_content = &adw::ToastOverlay {
                        adw::NavigationPage {
                            set_tag: Some("main-content"),
                            set_title: "Podcasts",

                            #[wrap(Some)]
                            set_child = &adw::ToolbarView {
                                add_top_bar = &gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    model.miniplayer.widget(),
                                    gtk::Separator {
                                        add_css_class: "tahoe-shimmer-line",
                                        #[watch]
                                        set_visible: model.is_loading,
                                        set_halign: gtk::Align::Fill,
                                        inline_css: "min-height: 2px; ..."
                                    },
                                },

                                #[wrap(Some)]
                                #[name = "content_bin"]
                                set_content = &adw::Bin {
                                    set_child: model.pages_cache.get(&model.current_page_key).map(|c| c.widget())
                                }
                            },
                        }
                    },
                },

                // PAGE 2: The player view injected into the stack
                add_named[Some("player_page")] = model.player_page.widget(){}
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let action_sender = sender.clone();
        let worker_controller =
            ActionWorker::builder()
                .launch(())
                .connect_receiver(move |_parent_sender, output| match output {
                    ActionWorkerOutput::NotifyError(error) => {
                        action_sender
                            .clone()
                            .input(AppModelInput::NotifyError(error));
                    }
                    ActionWorkerOutput::StateChanged(state, episode_id) => {
                        action_sender
                            .clone()
                            .input(AppModelInput::ChangePlayBackState(state, episode_id));
                    }
                    ActionWorkerOutput::SetCurrentEpisode(id) => {
                        action_sender
                            .clone()
                            .input(AppModelInput::SetCurrentEpisode(id));
                    }
                    ActionWorkerOutput::RefreshAllViews => {
                        action_sender.clone().input(AppModelInput::RefreshShowsPage);
                    }
                    ActionWorkerOutput::DownloadFinished(episode_id) => {
                        action_sender
                            .clone()
                            .input(AppModelInput::DownloadFinished(episode_id));
                    }
                    ActionWorkerOutput::DownloadCancelled(episode_id) => {
                        action_sender
                            .clone()
                            .input(AppModelInput::DownloadCancled(episode_id));
                    }
                    ActionWorkerOutput::DownloadProgress { id, fraction } => {
                        action_sender
                            .clone()
                            .input(AppModelInput::DownloadProgress(id, fraction));
                    }
                    ActionWorkerOutput::PlayBackProgress(id, pos, remaining) => {
                        action_sender
                            .clone()
                            .input(AppModelInput::PlayBackProgress(id, pos, remaining));
                    }
                    ActionWorkerOutput::VolumeValue(val) => {
                        action_sender.clone().input(AppModelInput::VolumeValue(val));
                    }
                    ActionWorkerOutput::EpisodeDeleted(episode_id) => {
                        action_sender
                            .clone()
                            .input(AppModelInput::EpisodeDeleted(episode_id));
                    }
                    _ => {}
                });

        let main_menu_button = MainMenuButton::builder().launch(()).detach();

        let homepage =
            HomePage::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    HomPageOutPut::Subscribe(feed) => AppModelInput::Subscribe(feed),
                    HomPageOutPut::ToggleSideBar => AppModelInput::ToggleSidebar,
                    HomPageOutPut::TogglePlay(episode_id) => AppModelInput::TogglePlay(episode_id),
                    HomPageOutPut::NotifyError(error) => AppModelInput::NotifyError(error),
                    HomPageOutPut::RequestDownload(episode_id) => {
                        AppModelInput::RequestDownload(episode_id)
                    }
                    HomPageOutPut::CancleDownload(episode_id) => {
                        AppModelInput::CancleDownload(episode_id)
                    }
                });

        let mut initial_cache = HashMap::new();
        let key = NavigationPage::Home.to_key();

        initial_cache.insert(key.clone(), PageController::Home(homepage));

        let miniplayer =
            MiniPlayerModel::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    MiniplayerModelOutput::TogglePlay => AppModelInput::TogglePlayBack,
                    MiniplayerModelOutput::NotifyError(error) => AppModelInput::NotifyError(error),
                    MiniplayerModelOutput::SeekAudioPosition(pos_fraction) => {
                        AppModelInput::SeekAudioPosition(pos_fraction)
                    }
                    MiniplayerModelOutput::SetVolume(fraction) => {
                        AppModelInput::SetVolume(fraction)
                    }
                    MiniplayerModelOutput::RequestMute => AppModelInput::RequestMute,
                    MiniplayerModelOutput::RequestUnmute => AppModelInput::RequestUnmute,
                    MiniplayerModelOutput::RequestVolumeValue => AppModelInput::RequestVolumeValue,
                    MiniplayerModelOutput::Seekforward => AppModelInput::Seekforward,
                    MiniplayerModelOutput::SeekBakward => AppModelInput::SeekBakward,
                    MiniplayerModelOutput::ShowPlayerPage(player_page_view) => {
                        AppModelInput::ShowPlayerPage(player_page_view)
                    }
                });

        let player_page = PlayerPage::builder().launch(()).forward(
            sender.input_sender(),
            |message| match message {
                PlayerPageOutput::ClosePlayer => AppModelInput::ClosePlayer,
                PlayerPageOutput::NotifyError(error) => AppModelInput::NotifyError(error),
            },
        );

        let model = AppModel {
            is_sidebar_visible: true,
            main_menu_button,
            pages_cache: initial_cache,
            current_page_key: key,
            player_page,
            miniplayer,
            worker_controller,
            is_loading: false,
            show_full_player: false,
        };

        // Generates the correct modern auto-derived struct layout type
        let widgets = view_output!();

        Self::render_sidebar_list(&widgets, &sender);

        widgets
            .library
            .set_selection_mode(gtk::SelectionMode::Single);
        widgets.pages.set_selection_mode(gtk::SelectionMode::Single);

        widgets.library.unselect_all();
        widgets.pages.unselect_all();

        if let Some(row) = widgets
            .pages
            .first_child()
            .and_then(|w| w.dynamic_cast::<gtk::ListBoxRow>().ok())
        {
            widgets.pages.select_row(Some(&row));
        }

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
            AppModelInput::ToggleSidebar => {
                self.is_sidebar_visible = !self.is_sidebar_visible;
            }
            AppModelInput::StartLoading => {
                self.is_loading = true;
            }
            AppModelInput::StopLoading => {
                self.is_loading = false;
            }

            AppModelInput::ShowSearchPage => {
                Self::show_search_page(widgets, &sender);
            }
            AppModelInput::SelectPage(page) => {
                let key = page.to_key();

                // 2. Lazily instantiate missing pages
                if !self.pages_cache.contains_key(&key) {
                    let controller = match page {
                        NavigationPage::Search => {
                            let page_instance = SearchPage::builder().launch(()).forward(
                                sender.input_sender(),
                                |msg| match msg {
                                    SearchPageOutput::Subscribe(feed) => {
                                        AppModelInput::Subscribe(feed)
                                    }
                                    SearchPageOutput::UpdateISSearching(state) => {
                                        if state {
                                            AppModelInput::StartLoading
                                        } else {
                                            AppModelInput::StopLoading
                                        }
                                    }
                                    SearchPageOutput::TogglePlay(episode) => {
                                        AppModelInput::TogglePlay(episode)
                                    }
                                    SearchPageOutput::NotifyError(error) => {
                                        AppModelInput::NotifyError(error)
                                    }
                                    SearchPageOutput::RequestDownload(episode_id) => {
                                        AppModelInput::RequestDownload(episode_id)
                                    }
                                    SearchPageOutput::CancleDownload(episode_id) => {
                                        AppModelInput::CancleDownload(episode_id)
                                    }
                                },
                            );
                            PageController::Search(page_instance)
                        }
                        NavigationPage::Home => {
                            let page_instance = HomePage::builder().launch(()).forward(
                                sender.input_sender(),
                                |msg| match msg {
                                    HomPageOutPut::Subscribe(feed) => {
                                        AppModelInput::Subscribe(feed)
                                    }
                                    HomPageOutPut::ToggleSideBar => AppModelInput::ToggleSidebar,
                                    HomPageOutPut::TogglePlay(episode_id) => {
                                        AppModelInput::TogglePlay(episode_id)
                                    }
                                    HomPageOutPut::NotifyError(error) => {
                                        AppModelInput::NotifyError(error)
                                    }
                                    HomPageOutPut::RequestDownload(episode_id) => {
                                        AppModelInput::RequestDownload(episode_id)
                                    }
                                    HomPageOutPut::CancleDownload(episode_id) => {
                                        AppModelInput::CancleDownload(episode_id)
                                    }
                                },
                            );
                            PageController::Home(page_instance)
                        }
                        NavigationPage::New => {
                            let page_instance = NewPage::builder().launch(()).detach();
                            PageController::New(page_instance)
                        }
                        NavigationPage::Shows => {
                            let page_instance = ShowsPage::builder().launch(()).forward(
                                sender.input_sender(),
                                |msg| match msg {
                                    ShowsPageOutput::NotifyError(error) => {
                                        AppModelInput::NotifyError(error)
                                    }
                                    ShowsPageOutput::TogglePlay(id) => {
                                        AppModelInput::TogglePlay(id)
                                    }
                                    ShowsPageOutput::RequestDownload(episode_id) => {
                                        AppModelInput::RequestDownload(episode_id)
                                    }
                                    ShowsPageOutput::CancleDownload(episode_id) => {
                                        AppModelInput::CancleDownload(episode_id)
                                    }
                                },
                            );
                            PageController::Shows(page_instance)
                        }

                        NavigationPage::Downloads => {
                            let page_instance = DownloadsPage::builder().launch(()).forward(sender.input_sender(), |msg| match msg {
                                crate::pages::downloads::DownloadsPageOutput::TogglePlay(episode_id) => AppModelInput::TogglePlay(episode_id),
                                crate::pages::downloads::DownloadsPageOutput::NotifyError(error) => AppModelInput::NotifyError(error),
                                crate::pages::downloads::DownloadsPageOutput::RequestDeleteEpisode(episode_id) => AppModelInput::RequestDeleteEpisode(episode_id),
                                crate::pages::downloads::DownloadsPageOutput::StartLoading => AppModelInput::StartLoading,
                                crate::pages::downloads::DownloadsPageOutput::StopLoading => AppModelInput::StopLoading,
                            });
                            PageController::Downloads(page_instance)
                        }
                        _ => return,
                    };

                    self.pages_cache.insert(key.clone(), controller);
                }

                // 3. Update active page and UI view
                self.current_page_key = key;
                if let Some(cached_page) = self.pages_cache.get(&self.current_page_key) {
                    widgets.content_bin.set_child(Some(cached_page.widget()));
                }
            }

            AppModelInput::SetSidebarCollapsed(_is_collapsed) => {}

            AppModelInput::NotifyError(error) => {
                let toast = adw::Toast::builder()
                    .title(error)
                    .action_name("Action")
                    .build();
                widgets.toast_overlay.add_toast(toast);
            }
            AppModelInput::Subscribe(feed) => {
                sender.input(AppModelInput::StartLoading);
                self.worker_controller
                    .emit(ActionWorkerInput::Subscirbe(feed));
            }
            AppModelInput::TogglePlay(id) => {
                println!("Streaming: {:?}", id);
                self.worker_controller
                    .emit(ActionWorkerInput::Execute(Action::TogglePlay(id)));
            }
            AppModelInput::ChangePlayBackState(state, episode_id) => {
                self.miniplayer
                    .emit(MiniplayerModelInput::ChangePlayBackState(state));
                self.player_page
                    .emit(PlayerPageInput::ChangePlayBackState(state));
                for (_, page) in &self.pages_cache {
                    page.notify_playing_state(episode_id, state);
                }
            }
            AppModelInput::SetCurrentEpisode(id) => {
                self.miniplayer
                    .emit(MiniplayerModelInput::SetCurrentEpisode(id));
                self.player_page
                    .emit(PlayerPageInput::SetCurrentEpisode(id));
                for (_, page) in &self.pages_cache {
                    page.notify_current_episode(id);
                }
            }
            AppModelInput::TogglePlayBack => {
                self.worker_controller
                    .emit(ActionWorkerInput::TogglePlayBack);
            }
            AppModelInput::RefreshShowsPage => {
                sender.input(AppModelInput::StopLoading);
                if let Some(PageController::Shows(shows_controller)) =
                    self.pages_cache.get(&self.current_page_key)
                {
                    shows_controller.sender().emit(ShowsPageInput::FetchShows);
                }
            }

            AppModelInput::None => {}
            AppModelInput::RequestDownload(episode_id) => {
                self.worker_controller
                    .emit(ActionWorkerInput::DownloadEpisode(episode_id));
            }
            AppModelInput::CancleDownload(episode_id) => {
                self.worker_controller
                    .emit(ActionWorkerInput::CancelDownload(episode_id));
            }
            AppModelInput::DownloadStarted(episode_id) => {
                for (_, page) in &self.pages_cache {
                    page.notify_download_started(episode_id);
                }
            }
            AppModelInput::DownloadCancled(episode_id) => {
                println!("Download of {:?} Cancled", episode_id);
            }
            AppModelInput::DownloadProgress(episode_id, fraction) => {
                for (_, page) in &self.pages_cache {
                    page.notify_download_progress(episode_id, fraction);
                }
            }
            AppModelInput::DownloadFinished(episode_id) => {
                for (_, page) in &self.pages_cache {
                    page.notify_download_finished(episode_id);
                }
            }
            AppModelInput::PlayBackProgress(episode_id, pos, remaining) => {
                self.miniplayer
                    .emit(MiniplayerModelInput::UpdateProgress(pos));

                self.player_page
                    .emit(PlayerPageInput::UpdateProgress(pos, remaining));
                for (_, page) in &self.pages_cache {
                    page.notify_playback_progress(episode_id, pos, remaining);
                }
            }
            AppModelInput::SeekAudioPosition(fraction) => {
                self.worker_controller
                    .emit(ActionWorkerInput::SeekAudioPosition(fraction));
            }
            AppModelInput::SetVolume(fraction) => {
                self.worker_controller
                    .emit(ActionWorkerInput::SetVolume(fraction));
            }
            AppModelInput::RequestMute => {
                self.worker_controller.emit(ActionWorkerInput::RequestMute);
            }
            AppModelInput::RequestUnmute => {
                self.worker_controller
                    .emit(ActionWorkerInput::RequestUnmute);
            }
            AppModelInput::RequestVolumeValue => {
                self.worker_controller.emit(ActionWorkerInput::GetVolume);
            }
            AppModelInput::VolumeValue(val) => {
                self.miniplayer.emit(MiniplayerModelInput::VolumeValue(val));

                self.player_page.emit(PlayerPageInput::VolumeValue(val));
            }
            AppModelInput::Seekforward => {
                self.worker_controller.emit(ActionWorkerInput::SeekFoward);
            }
            AppModelInput::SeekBakward => {
                self.worker_controller.emit(ActionWorkerInput::SeekBackward);
            }
            AppModelInput::RequestDeleteEpisode(episode_id) => {
                self.worker_controller
                    .emit(ActionWorkerInput::DeleteEpisode(episode_id));
            }
            AppModelInput::EpisodeDeleted(episode_id) => {
                for (_, page) in &self.pages_cache {
                    page.notify_episode_deleted(episode_id);
                }
            }
            AppModelInput::ShowPlayerPage(player_page_view) => {
                self.show_full_player = true;
            }
            AppModelInput::ClosePlayer => {
                self.show_full_player = false;
            }
        }

        self.update_view(widgets, sender);
    }
}
