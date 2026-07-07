use gtk::gio::prelude::FileExt;
use podcasts_data::Episode;
use adw::prelude::*;
use relm4::{
    FactorySender,
    factory::{DynamicIndex, FactoryComponent},
};

pub struct EpisodeListItem {
    episode: Episode,
    texture: Option<adw::gdk::Texture>,
}

#[derive(Debug)]
pub enum EpisodeListItemInput {
    ImageDownloaded(Option<adw::gdk::Texture>),
}

#[derive(Debug)]
pub enum EpisodeListItemOutput {}

#[derive(Debug)]
pub enum EpisodeListItemCmdInput {
    DownloadImage(Option<adw::gdk::Texture>),
}

#[relm4::factory(pub)]
impl FactoryComponent for EpisodeListItem {
    type Init = Episode;
    type Input = EpisodeListItemInput;
    type Output = EpisodeListItemOutput;
    type CommandOutput = EpisodeListItemCmdInput;
    type ParentWidget = gtk::ListBox;

    fn init_model(episode: Self::Init, _index: &DynamicIndex, sender: FactorySender<Self>) -> Self {
        // let clone = episode.clone();

        // if let Some(image_url_ref) = clone.image_uri() {
        //     let image_url = image_url_ref.to_string();

        //     sender.oneshot_command(async move {
        //         let texture_res = tokio::task::spawn_blocking(move || {
        //             let load_image = || -> Option<gtk::gdk::Texture> {
        //                 let file = gtk::gio::File::for_uri(&image_url);
        //                 let (glib_bytes, _) = file.load_bytes(gtk::gio::Cancellable::NONE).ok()?;
        //                 gtk::gdk::Texture::from_bytes(&glib_bytes).ok()
        //             };

        //             load_image()
        //         })
        //         .await;

        //         let downloaded_texture = match texture_res {
        //             Ok(Some(texture)) => Some(texture),
        //             _ => None,
        //         };

        //         EpisodeListItemCmdInput::DownloadImage(downloaded_texture)
        //     });
        // }

        Self {
            episode,
            texture: None,
        }
    }

    view! {
        adw::ActionRow{
            set_title: self.episode.title()
        }
    }
}
