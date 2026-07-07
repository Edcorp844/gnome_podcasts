use podcasts_data::dbqueries;
use podcasts_data::discovery::{ALL_PLATFORM_IDS, FoundPodcast, SearchError, search};
use relm4::adw::prelude::*;
use relm4::prelude::*;

use crate::components::found_podcast_ui::{FoundCardOutput, FoundPodcastsCard};

pub struct HomePage {
    pub podcasts: FactoryVecDeque<FoundPodcastsCard>,
    pub is_loading: bool,
}

#[derive(Debug)]
pub enum HomePageCommand {
    Podcasts(Result<Vec<FoundPodcast>, SearchError>),
}

#[derive(Debug)]
pub enum HomePageInput {
    FetchPodcasts,
    PodcastsLoaded(Result<Vec<FoundPodcast>, SearchError>),
    // Subscribe(String),
}

#[derive(Debug)]
pub enum HomPageOutPut {
    ToggleSideBar,
    Subscribe(String),
}

#[relm4::component(pub)]
impl Component for HomePage {
    type Init = ();
    type Input = HomePageInput;
    type Output = HomPageOutPut;
    type CommandOutput = HomePageCommand;

    view! {
        adw::NavigationPage {
            set_title: "Podcasts",

            #[wrap(Some)]
            set_child = &adw::NavigationPage {

                    #[wrap(Some)]
                    set_child = &adw::ToolbarView {

                        #[wrap(Some)]
                        set_content = &gtk::Box{
                            set_orientation: gtk::Orientation::Vertical,
                            gtk::Separator {
                                add_css_class: "tahoe-shimmer-line",
                                #[watch]
                                set_visible: model.is_loading,
                                set_halign: gtk::Align::Fill,
                                inline_css: " min-height: 2px;
                                    border: none;  background: linear-gradient(90deg, 
                                        rgba(0, 122, 255, 0) 0%, 
                                        #007AFF 25%, 
                                        #AF52DE 50%, 
                                        #FF2D55 75%, 
                                        rgba(255, 45, 85, 0) 100%
                                    );
                                    background-size: 200% 100%; animation: s from { background-position: 0% 0%; }
                                    to { background-position: 200% 0%; }"

                            },

                            gtk::ScrolledWindow {
                                set_vexpand : true,
                                set_hscrollbar_policy: gtk::PolicyType::Never,

                                adw::Clamp {
                                    set_maximum_size: 1400,
                                    set_tightening_threshold: 1000,

                                    gtk::Box {
                                        set_orientation: gtk::Orientation::Vertical,
                                        gtk::Label {
                                            set_margin_top: 40,
                                            set_margin_horizontal: 20,
                                            set_label: "Home",
                                            set_halign:gtk::Align::Start,

                                            add_css_class: "title-1"
                                        },

                                        #[local_ref]
                                        podcast_grid -> gtk::FlowBox {
                                            set_margin_all: 20,
                                            #[watch]
                                            set_visible: !model.is_loading,
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

        }
    }

    fn init(
        _worker_sender: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // Build your customized FlowBox layout container manually
        let grid = gtk::FlowBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .column_spacing(20)
            .row_spacing(40)
            .homogeneous(true)
            .build();

        // Attach your dynamic width column calculating callback hook
        let last_width = std::cell::Cell::new(0);
        grid.add_tick_callback(move |grid, _| {
            let width = grid.width();
            if width != last_width.get() {
                last_width.set(width);
                let columns = match width {
                    0..=500 => 2,
                    501..=800 => 3,
                    801..=1100 => 4,
                    _ => 5,
                };
                grid.set_min_children_per_line(columns);
                grid.set_max_children_per_line(columns);
            }
            gtk::glib::ControlFlow::Continue
        });

        let model = HomePage {
            podcasts: FactoryVecDeque::builder().launch(grid).forward(
                sender.output_sender(),
                |msg| match msg {
                    FoundCardOutput::Subscribe(feed) => HomPageOutPut::Subscribe(feed),
                },
            ),
            is_loading: true,
        };

        // Resolve the reference for the local_ref macro parameter mapping step
        let podcast_grid = model.podcasts.widget();
        let widgets = view_output!();

        sender.input(HomePageInput::FetchPodcasts);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            HomePageInput::FetchPodcasts => {
                let search_texts = vec!["uganda", "news", "footboall", "Jesus", "phaneroo"];

                for text in search_texts {
                    // Remove '|text|'—the variable is automatically captured by the 'move' keyword
                    sender.oneshot_command(async move {
                        for id in ALL_PLATFORM_IDS {
                            match dbqueries::set_discovery_setting(id, true) {
                                Err(e) => {
                                    println!("Error settings: {}", e);
                                }
                                Ok(_) => {}
                            }
                        }

                        HomePageCommand::Podcasts(search(&text).await)
                    });
                }
            }

            // Captures background thread work payload safely
            HomePageInput::PodcastsLoaded(podcasts) => {
                match podcasts {
                    Ok(data) => {
                        let mut guard = self.podcasts.guard();
                        //guard.clear(); // Flush old elements cleanly
                        for podcast in data {
                            guard.push_back(podcast);
                        }
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
                self.is_loading = false;
            }
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _: &Self::Root,
    ) {
        match message {
            HomePageCommand::Podcasts(data) => sender.input(HomePageInput::PodcastsLoaded(data)),
        }
    }
}
