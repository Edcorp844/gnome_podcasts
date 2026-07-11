use crate::action::Action;
use crate::app_navigation_ext::NavigationPage;
use crate::app_navigation_ext::PageController;
use crate::components::miniplayer::MiniPlayerModel;

use crate::components::miniplayer::MiniplayerModelInput;
use crate::components::miniplayer::MiniplayerModelOutput;
use crate::pages::home::HomPageOutPut;
use crate::pages::home::HomePage;
use crate::pages::new::NewPage;
use crate::pages::search::SearchPage;
use crate::pages::search::SearchPageOutput;
use crate::pages::shows::ShowsPage;
use crate::pages::shows::ShowsPageInput;
use crate::pages::shows::ShowsPageOutput;
use crate::workers::action_worker::service::ActionWorker;
use crate::workers::action_worker::service::ActionWorkerInput;
use crate::workers::action_worker::service::ActionWorkerOutput;

use adw::gio;
use gst_play::PlayState;
use relm4::ComponentParts;
use relm4::RelmIterChildrenExt;
use relm4::adw::prelude::*;
use relm4::prelude::*;

use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;

use podcasts_data::{EpisodeId, EpisodeWidgetModel, ShowId};

pub struct AppModel {
    pub is_sidebar_visible: bool,
    pages_cache: HashMap<String, PageController>,
    current_page_key: String,
    miniplayer: Controller<MiniPlayerModel>,
    worker_controller: Controller<ActionWorker>,
    pub is_loading: bool,
    pub updating: bool,
    pub active_show_id: Option<ShowId>,
    pub active_show_title: String,
    on_search_page: bool,
    settings: RefCell<Option<gio::Settings>>,
    inhibit_cookie: RefCell<u32>,
    todo_unsub_ids: RefCell<HashSet<ShowId>>,
    undo_marked_ids: RefCell<Vec<ShowId>>,
}

#[derive(Debug)]
pub enum AppModelInput {
    ToggleSidebar,
    StartLoading,
    StopLoading,
    ShowSearchPage,
    TogglePlayBack,
    SelectPage(NavigationPage),
    SetSidebarCollapsed(bool),
    RefreshShowsPage,
    HandleVolumeChange(f64),
    StreamEpisode(EpisodeId),
    NotifyError(String),
    Subscribe(String),
    ChangePlayBackState(PlayState),
    SetCurrentEpisode(EpisodeId),
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
            set_content = &adw::OverlaySplitView {
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


                        pack_end = &gtk::Button {
                            set_icon_name: "open-menu-symbolic",
                            set_tooltip: "Main Menu",
                            add_css_class: "flat",
                            connect_clicked[sender] => move |_| {
                                //let _ = sender.output(AudioBibleOutput::ToggleSidebar);
                            }
                        }
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
                set_content = &adw::ToastOverlay{

                    adw::NavigationPage {
                        set_tag: Some("main-content"),
                        set_title: "Podcasts",

                        #[wrap(Some)]
                        set_child = &adw::ToolbarView {

                            add_top_bar = &gtk::Box{
                                set_orientation: gtk::Orientation::Vertical,
                                model.miniplayer.widget(),
                                gtk::Separator {
                                    add_css_class: "tahoe-shimmer-line",
                                    #[watch]
                                    set_visible: model.is_loading,
                                    set_halign: gtk::Align::Fill,
                                    inline_css: " min-height: 2px;
                                        background: linear-gradient(90deg, 
                                            alpha(var(--accent-color), 0) 0%, 
                                            var(--accent-color) 25%, 
                                            var(--accent-bg-color) 75%, 
                                            alpha(var(--accent-bg-color), 0) 100%
                                        );
                                        background-size: 200% 100%; 
                                        animation: shimmer-flow 1.5s infinite linear;"
                                },
                            },

                            #[wrap(Some)]
                            #[name = "content_bin"]
                            set_content =&adw::Bin {
                                set_child: model.pages_cache.get(&model.current_page_key).map(|c| c.widget())
                            }

                        },
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
                    ActionWorkerOutput::StateChanged(state) => {
                        action_sender
                            .clone()
                            .input(AppModelInput::ChangePlayBackState(state));
                    }
                    ActionWorkerOutput::SetCurrentEpisode(id) => {
                        action_sender
                            .clone()
                            .input(AppModelInput::SetCurrentEpisode(id));
                    }
                    ActionWorkerOutput::RefreshAllViews => {
                        action_sender.clone().input(AppModelInput::RefreshShowsPage);
                    }
                    _ => {}
                });

        // Create HomePage and subscribe it to worker output
        let homepage =
            HomePage::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    HomPageOutPut::Subscribe(feed) => AppModelInput::Subscribe(feed),
                    _ => AppModelInput::None,
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
                });

        let model = AppModel {
            is_sidebar_visible: true,
            pages_cache: initial_cache,
            current_page_key: key,
            miniplayer,
            worker_controller,
            is_loading: false,
            updating: false,
            on_search_page: false,
            active_show_id: None,
            active_show_title: String::new(),
            settings: RefCell::new(None),
            inhibit_cookie: RefCell::new(0),
            todo_unsub_ids: RefCell::new(HashSet::default()),
            undo_marked_ids: RefCell::new(vec![]),
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
                                    SearchPageOutput::StreamEpisode(episode) => {
                                        AppModelInput::StreamEpisode(episode)
                                    }
                                },
                            );
                            PageController::Search(page_instance)
                        }
                        NavigationPage::Home => {
                            let page_instance = HomePage::builder().launch(()).detach();
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
                                    ShowsPageOutput::NotifyError(err) => {
                                        AppModelInput::NotifyError(err)
                                    }
                                    ShowsPageOutput::StreamEpisode(id) => {
                                        AppModelInput::StreamEpisode(id)
                                    }
                                },
                            );
                            PageController::Shows(page_instance)
                        }
                        _ => return, // Early exit if page is unsupported to avoid broken state
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
            AppModelInput::HandleVolumeChange(new_vol) => {}
            AppModelInput::NotifyError(error) => {
                println!("Error: recieved: {}", error);
                let toast = adw::Toast::builder()
                    .title(error)
                    .action_name("Subsricption")
                    .build();
                widgets.toast_overlay.add_toast(toast);
            }
            AppModelInput::Subscribe(feed) => {
                sender.input(AppModelInput::StartLoading);
                self.worker_controller
                    .emit(ActionWorkerInput::Subscirbe(feed));
            }
            AppModelInput::StreamEpisode(id) => {
                println!("Streaming: {:?}", id);
                self.worker_controller
                    .emit(ActionWorkerInput::Execute(Action::StreamEpisode(id)));
            }
            AppModelInput::ChangePlayBackState(state) => {
                self.miniplayer
                    .emit(MiniplayerModelInput::ChangePlayBackState(state));
            }
            AppModelInput::SetCurrentEpisode(id) => {
                self.miniplayer
                    .emit(MiniplayerModelInput::SetCurrentEpisode(id));
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
        }

        self.update_view(widgets, sender);
    }
}
