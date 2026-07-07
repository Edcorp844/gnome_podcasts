use adw::prelude::*;
use podcasts_data::{
    Show, ShowId,
    dbqueries::{self, ShowFilter},
    errors::DataError,
};
use relm4::{Component, ComponentParts, ComponentSender, prelude::*};

use crate::{
    components::show_card::{ShowCard, ShowCardOutput},
    pages::show::ShowPage,
};

pub struct ShowsPage {
    shows: FactoryVecDeque<ShowCard>,
    open_show_pages: Vec<Controller<ShowPage>>,
    is_loading: bool,
}

#[derive(Debug)]
pub enum ShowsPageInput {
    FetchShows,
    ShowsLoaded(Result<Vec<Show>, DataError>),
    GotoShow(ShowId),
}

#[derive(Debug)]
pub enum ShowsPageOutput {}

#[derive(Debug)]
pub enum ShowsPageCommand {
    //Shows(data),
}

#[relm4::component(pub)]
impl Component for ShowsPage {
    type Init = ();
    type Input = ShowsPageInput;
    type Output = ShowsPageOutput;
    type CommandOutput = ShowsPageCommand;

    fn init(
        _worker_sender: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
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

        let model = ShowsPage {
            shows: FactoryVecDeque::builder()
                .launch(grid)
                .forward(sender.input_sender(), |msg| match msg {
                    ShowCardOutput::GotoShow(show) => ShowsPageInput::GotoShow(show),
                }),
                open_show_pages: Vec::new(),
            is_loading: true,
        };

        let show_grid = model.shows.widget();

        let widgets = view_output!();

        sender.input(ShowsPageInput::FetchShows);

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
            ShowsPageInput::FetchShows => {
                let filter = ShowFilter {
                    any_downloaded: None,
                    completed: None,
                    title_or_description: None,
                    reverse_order: false,
                };
                let data = dbqueries::get_podcasts_filter(&[], &filter);
                sender.input(ShowsPageInput::ShowsLoaded(data));
            }

            ShowsPageInput::ShowsLoaded(shows) => {
                match shows {
                    Ok(data) => {
                        let mut guard = self.shows.guard();
                        guard.clear();
                        for show in data {
                            guard.push_back(show);
                        }
                    }
                    Err(e) => {
                        println!("Error loading shows: {}", e.to_string());
                    }
                }
                self.is_loading = false;
            }

            ShowsPageInput::GotoShow(id) => {
                let show_page = ShowPage::builder().launch(id.clone()).detach();

                widgets.nav_view.push(show_page.widget());
                self.open_show_pages.push(show_page);
            }
        }

        self.update_view(widgets, sender.clone());
    }

    view! {
        adw::NavigationPage {
            set_title: "Podcasts",

            #[wrap(Some)]
            #[name = "nav_view"]
            set_child = &adw::NavigationView{
                add=&adw::NavigationPage {

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
                                            set_label: "Shows",
                                            set_halign:gtk::Align::Start,

                                            add_css_class: "title-1"
                                        },

                                        #[local_ref]
                                        show_grid -> gtk::FlowBox {
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
    }
}
