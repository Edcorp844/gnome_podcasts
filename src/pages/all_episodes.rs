use std::collections::BTreeMap;

use adw::prelude::*;
use chrono::Datelike;
use podcasts_data::Episode;
use relm4::{Component, prelude::*};

use crate::components::episode_group::GroupedEpisodes;

#[derive(Debug)]
pub struct AllEpisodesPage {
    groups: Vec<Controller<GroupedEpisodes>>,
}

#[derive(Debug)]
pub enum AllEpisodesPageinput {
    SetEpisodes(Vec<Episode>),
}

#[relm4::component(pub)]
impl Component for AllEpisodesPage {
    type Init = ();
    type Input = AllEpisodesPageinput;
    type Output = ();
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
        sender: ComponentSender<Self>,
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
    root: &Self::Root,
) {
    match message {
        AllEpisodesPageinput::SetEpisodes(mut episodes) => {
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
                    .launch((format!("{}",*year), episodes.clone()))
                    .detach();
                widgets.group_parent.append(group.widget());
                self.groups.push(group); // <-- keep it alive
            }
        }
    }
}
   
}
