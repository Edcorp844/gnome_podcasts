use adw::prelude::*;

use podcasts_data::{
    dbqueries,
    discovery::{ALL_PLATFORM_IDS, FoundPodcast, SearchError, search},
};
use relm4::{Component, ComponentParts, ComponentSender, prelude::*};

use crate::components::podcast_search_results::{PodcastResults, PodcastResultsInput};

// 1. Define the possible pages you want to visit across your app
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageTarget {
    MainPodcastsList,
    PodcastDetails,
    EpisodeDescription,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultCategory {
    Podcast,
    library,
}

pub struct SearchPage {
    pub current_page: PageTarget,
    result_category: ResultCategory,
    podcast_results_widget: Controller<PodcastResults>
}

#[derive(Debug)]
pub enum SearchPageInput {
    PushPage(PageTarget),
    PopPage,
    SwitchResultCategory(ResultCategory),
    PodcastsLoaded(Result<Vec<FoundPodcast>, SearchError>),
    UpdateQuery(String),
    TriggerSearch(String),
}

#[derive(Debug)]
pub enum SearchPageOutput {}

#[derive(Debug)]
pub enum SearchPageCmdInput {
    Podcasts(Result<Vec<FoundPodcast>, SearchError>),
}

#[relm4::component(pub)]
impl Component for SearchPage {
    type Init = ();
    type Input = SearchPageInput;
    type Output = SearchPageOutput;
    type CommandOutput = SearchPageCmdInput;

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let podcast_results_widget = PodcastResults::builder().launch(()).detach();
        let model = Self {
            current_page: PageTarget::MainPodcastsList,
            result_category: ResultCategory::Podcast,
            podcast_results_widget
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input, // Keeps original ownership here
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        // FIX: Match against a reference to prevent partial moves
        match &message {
            SearchPageInput::PodcastsLoaded(data) => match data {
                Ok(data) => {
                   self.podcast_results_widget.emit(PodcastResultsInput::Results(data.clone()));
                }
                Err(_error) => {}
            },
            SearchPageInput::TriggerSearch(text) => {
                self.podcast_results_widget.emit(PodcastResultsInput::SearchBegan);
                let search_text = text.clone();
                println!("Searching: {search_text}");
                sender.oneshot_command(async move {
                    for id in ALL_PLATFORM_IDS {
                        match dbqueries::set_discovery_setting(id, true) {
                            Err(e) => {
                                println!("Error settings: {}", e);
                            }
                            Ok(_) => {}
                        }
                    }
                    // Use the cloned thread-safe string here
                    SearchPageCmdInput::Podcasts(search(&search_text).await)
                });
            }
            SearchPageInput::UpdateQuery(_text) => {}
            SearchPageInput::PushPage(target) => {
                self.current_page = *target; // Copy the enum value out

                match target {
                    PageTarget::PodcastDetails => {
                        widgets.nav_view.push_by_tag("podcast-details-page");
                    }
                    PageTarget::EpisodeDescription => {
                        widgets.nav_view.push_by_tag("episode-desc-page");
                    }
                    PageTarget::MainPodcastsList => {
                        widgets.nav_view.push_by_tag("root-podcasts-page");
                    }
                }
            }
            SearchPageInput::PopPage => {
                widgets.nav_view.pop();
            }
            SearchPageInput::SwitchResultCategory(category)=>{
                self.result_category = *category;
            }
        }

        // Now message is untouched and can be safely consumed here
        self.update(message, sender, root);
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _: &Self::Root,
    ) {
        match message {
            SearchPageCmdInput::Podcasts(data) => {
                sender.input(SearchPageInput::PodcastsLoaded(data))
            }
        }
    }

    view! {
        adw::NavigationPage {
            set_title: "Explore",
            set_tag: Some("main-navigator-root"),

            #[wrap(Some)]
            #[name = "nav_view"]
            set_child = &adw::NavigationView {

                // --- PAGE 1: Root List View ---
                #[name = "root_page"]
                add = &adw::NavigationPage {
                    set_title: "Podcasts",
                    set_tag: Some("root-podcasts-page"),

                    #[wrap(Some)]
                    set_child = &adw::ToolbarView {

                        add_top_bar = &adw::HeaderBar {
                            set_show_start_title_buttons: false,
                            set_show_end_title_buttons: false,

                            // FIX: Centering wrapper structure explicitly managed via the HeaderBar's placement layer
                            #[wrap(Some)]
                            set_title_widget = &gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_spacing: 6,
                                set_hexpand: true,
                                // FIX: Changed from Center to Fill so the bar expands to its bounding limits
                                set_halign: gtk::Align::Center,
                                set_valign: gtk::Align::Center,

                                // FIX: Forcing explicit dimensions via size request constraints
                                set_size_request: (360, -1),

                                gtk::SearchEntry {
                                    set_placeholder_text: Some("Search for podcasts..."),
                                    set_hexpand: true,
                                    set_halign: gtk::Align::Fill,

                                    // TRIGGER A: Fires on every single character keystroke (Live Search)
                                    connect_search_changed[sender] => move |entry| {
                                        let text = entry.text().to_string();
                                        sender.input(SearchPageInput::UpdateQuery(text));
                                    },

                                    // TRIGGER B: Fires only when pressing Enter (Debounced/Manual Search)
                                    connect_activate[sender] => move |entry| {
                                        let text = entry.text().to_string();
                                        sender.input(SearchPageInput::TriggerSearch(text));
                                    }
                                },

                            },

                            pack_end=&adw::ToggleGroup {
                                    // Keeps item button segments uniform in dimension
                                    set_homogeneous: true,
                                    //add_css_class: "round",

                                    // Use a string matching the 'name' property below to dictate the default selection
                                    set_active_name: Some("podcasts"),

                                    // Track layout transitions when a user toggles the items
                                    connect_active_name_notify[sender] => move |group| {
                                        if let Some(active_tab) = group.active_name() {
                                            match active_tab.as_str() {
                                                "podcasts" => {  sender.input(SearchPageInput::SwitchResultCategory(ResultCategory::Podcast)) }
                                                "library" => { sender.input(SearchPageInput::SwitchResultCategory(ResultCategory::Podcast))  }
                                                _ => {}
                                            }
                                        }
                                    },

                                    // --- Tab Selection Options ---
                                    add = adw::Toggle {
                                        set_name: Some("podcasts"),
                                        set_label: Some("Podcasts"),

                                    },

                                    add = adw::Toggle {
                                        set_name: Some("library"),
                                        set_label: Some("Library"),

                                    }
                                }
                        },

                       #[wrap(Some)]
                        set_content = model.podcast_results_widget.widget(),
                    }
                }
            },
        }
    }
}
