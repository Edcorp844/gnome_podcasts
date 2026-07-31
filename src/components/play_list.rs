use gtk::prelude::*;
use podcasts_data::{EpisodeId, dbqueries};
use relm4::{Component, ComponentParts, prelude::*};

use crate::{
    components::playlist_episode_list_item::PlayListEpisodeListItem, util::play_list::PlayList,
};

#[derive(Debug)]
pub struct PlayListComponent {
    play_list: Option<PlayList>,
    episodes: FactoryVecDeque<PlayListEpisodeListItem>,
}

#[derive(Debug)]
pub enum PlayListComponentInput {
    UpdatePlaylist(Vec<EpisodeId>, Option<usize>),
}

#[relm4::component(pub)]
impl Component for PlayListComponent {
    type Init = ();
    type Input = PlayListComponentInput;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::Box{
            set_orientation: gtk::Orientation::Vertical,

             #[local_ref]
            episodes_container -> gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                 set_halign: gtk::Align::Start,
                set_valign: gtk::Align::Start,
                #[watch]
                set_visible: !model.episodes.is_empty(),
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: relm4::prelude::ComponentSender<Self>,
    ) -> relm4::prelude::ComponentParts<Self> {
        let episodes_parent = gtk::Box::builder().build();
        let model = PlayListComponent {
            play_list: None,
            episodes: FactoryVecDeque::builder().launch(episodes_parent).detach(),
        };

        let episodes_container = model.episodes.widget();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        match message {
            PlayListComponentInput::UpdatePlaylist(episode_ids, _) => {
                let mut guard = self.episodes.guard();
                guard.clear();

                for id in episode_ids.iter() {
                    match dbqueries::get_episode_from_id(id.clone()) {
                        Ok(episode) => {
                            guard.push_back(episode);
                        }
                        Err(_) => {}
                    }
                }
            }
        }
    }
}
