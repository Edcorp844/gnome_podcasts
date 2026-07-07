use adw::prelude::*;
use podcasts_data::{
    Episode, Show, ShowId,
    dbqueries::{self, EpisodeFilter},
    errors::DataError,
};
use relm4::{Component, ComponentParts, ComponentSender, prelude::*};

use crate::components::episode_list_item::EpisodeListItem;

pub struct ShowPage {
    show: Option<Show>,
    episodes: FactoryVecDeque<EpisodeListItem>,
    show_image_texture: Option<adw::gdk::Texture>,
    load_error: Option<String>,
}

#[derive(Debug)]
pub enum ShowPageInput {
    GetShow(ShowId),
    ShowGotten(Result<Show, DataError>),
    FetchEpisodes,
    FetchEpisodesComplete(Result<Vec<Episode>, DataError>),
    ImageDownloaded(Option<adw::gdk::Texture>),
}

#[derive(Debug)]
pub enum ShowPageCmdInput {
    DownloadImage(Option<adw::gdk::Texture>),
}

#[relm4::component(pub)]
impl Component for ShowPage {
    type Init = ShowId;
    type Input = ShowPageInput;
    type Output = ();
    type CommandOutput = ShowPageCmdInput;

    view! {
        adw::NavigationPage {
            #[wrap(Some)]
            set_child = &adw::ToolbarView {

                add_top_bar = &adw::HeaderBar {
                    set_show_title: false,
                },

                #[wrap(Some)]
                set_content = &gtk::ScrolledWindow {
                    set_vexpand: true,
                    set_hscrollbar_policy: gtk::PolicyType::Never,

                    adw::Clamp {
                        set_maximum_size: 1100,
                        set_tightening_threshold: 900,

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_margin_all: 32,
                            set_spacing: 24,



                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_spacing: 28,
                                set_valign: gtk::Align::Start,

                                // Cover Art Block Container
                                gtk::Overlay {
                                    set_height_request: 350,
                                    set_width_request: 350,
                                    set_halign: gtk::Align::Start,
                                    set_valign: gtk::Align::Start,

                                    #[wrap(Some)]
                                    set_child = &gtk::Box {
                                        set_orientation: gtk::Orientation::Vertical,
                                        set_hexpand: true,
                                        set_vexpand: true,
                                        set_halign: gtk::Align::Fill,
                                        set_valign: gtk::Align::Fill,

                                        inline_css: "
                                            background-color: mix(@window_bg_color, @card_fg_color, 0.1);
                                            border-radius: 16px;
                                            box-shadow: 0 12px 28px rgba(0, 0, 0, 0.15);
                                            border: 1px solid rgba(255, 255, 255, 0.05);
                                        ",

                                        gtk::Label {
                                            #[watch]
                                            set_label: &{
                                                model.show.as_ref()
                                                    .map(|s| s.title().trim().chars().take(2).collect::<String>().to_uppercase())
                                                    .unwrap_or_default()
                                            },
                                            add_css_class: "title-large",
                                            set_hexpand: true,
                                            set_vexpand: true,
                                            set_halign: gtk::Align::Center,
                                            set_valign: gtk::Align::Center,
                                            inline_css: "color: @dim_label_opacity; opacity: 0.25; font-weight: 800;",
                                        }
                                    },

                                    add_overlay = &gtk::Picture {
                                        #[watch]
                                        set_paintable: model.show_image_texture.as_ref().map(|t| t.upcast_ref::<adw::gdk::Paintable>()),
                                        #[watch]
                                        set_visible: model.show_image_texture.is_some(),

                                        set_hexpand: true,
                                        set_vexpand: true,
                                        set_halign: gtk::Align::Fill,
                                        set_valign: gtk::Align::Fill,
                                        set_content_fit: gtk::ContentFit::Cover,
                                        set_can_shrink: true,
                                        inline_css: "border-radius: 16px; overflow: hidden;",
                                    }
                                },

                                // Text Detail Layout Column
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 6,
                                    set_hexpand: true,
                                    set_valign: gtk::Align::Start,

                                    // Podcast Title Header
                                    gtk::Label {
                                        #[watch]
                                        set_label: model.show.as_ref().map(|s| s.title().trim()).unwrap_or("Loading Show..."),
                                        set_halign: gtk::Align::Start,
                                        set_wrap: true,
                                        set_wrap_mode: gtk::pango::WrapMode::WordChar,
                                        add_css_class: "title-1"
                                    },


                                    // Description Excerpt Block
                                    gtk::Label {
                                        #[watch]
                                        set_label: model.show.as_ref().map(|s| s.description()).map(|d| d.trim()).unwrap_or("No description available."),
                                        set_halign: gtk::Align::Start,
                                        set_wrap: true,
                                        set_max_width_chars: 75,
                                        set_lines: 3,
                                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                                        set_css_classes: &vec!["dimmed", "body"],
                                    },

                                    gtk::Separator { set_vexpand: true, add_css_class: "spacer" },
                                    gtk::Separator { set_vexpand: true, add_css_class: "spacer" },
                                    // Interactive Row Buttons Layout
                                    gtk::Box {
                                        set_orientation: gtk::Orientation::Horizontal,
                                        set_spacing: 12,
                                        set_halign: gtk::Align::Start,

                                        gtk::Button {
                                            set_label: "▶ Latest Episode",
                                            set_css_classes: &vec!["pill", "suggested-action"],
                                        },

                                        gtk::Separator { set_hexpand: true, add_css_class: "spacer" },

                                        gtk::Button {
                                            set_label: "+ Follow",
                                            add_css_class: "pill",
                                            inline_css: "
                                                background-color: rgba(255, 255, 255, 0.1); 
                                                color: white; 
                                                font-weight: 600; 
                                                padding: 8px 16px;
                                                border: none;
                                            ",
                                        },

                                        gtk::Button {
                                            set_icon_name: "view-more-symbolic",
                                            add_css_class: "flat",
                                            inline_css: "
                                                background-color: rgba(255, 255, 255, 0.1); 
                                                border-radius: 50%;
                                                padding: 8px;
                                            ",
                                        },
                                    }
                                }
                            },

                            // --- EPISODES HEADER LIST DIVISION ---
                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_halign: gtk::Align::Start,
                                set_spacing: 4,
                                inline_css: "margin-top: 16px;",

                                gtk::Label {
                                    set_label: "Episodes",
                                    add_css_class: "title-2",
                                    inline_css: "font-weight: 700; font-size: 1.4rem;",
                                    set_valign: gtk::Align::Center,
                                },

                                gtk::Image {
                                    set_icon_name: Some("go-next-symbolic"),
                                    inline_css: "opacity: 0.7; margin-left: 4px;",
                                    set_valign: gtk::Align::Center,
                                }
                            },

                            #[local_ref]
                            episodes_container -> gtk::ListBox {
                                add_css_class: "boxed-list",

                            }

                        }
                    }
                }
            }
        }
    }

    fn init(
        show_id: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let episodes_parent = gtk::ListBox::builder().build();
        let model = Self {
            episodes: FactoryVecDeque::builder().launch(episodes_parent).detach(),
            show: None,
            show_image_texture: None,
            load_error: None,
        };

        let episodes_container = model.episodes.widget();

        let widgets = view_output!();

        sender.input(ShowPageInput::GetShow(show_id));
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            ShowPageInput::GetShow(show_id) => {
                // DIAGNOSTIC: confirms the id actually arriving here matches
                // what you expect. If this never prints, the input isn't
                // reaching this component at all (e.g. the page shown on
                // screen is a different/stale instance) — a very different
                // problem than a failed DB lookup.
                println!("ShowPage: fetching show_id = {:?}", show_id);

                let show_result = dbqueries::get_podcast_from_id(show_id);

                // DIAGNOSTIC: confirms whether the lookup itself succeeded.
                match &show_result {
                    Ok(show) => println!("ShowPage: fetched show '{}'", show.title()),
                    Err(e) => println!("ShowPage: fetch FAILED: {e}"),
                }

                sender.input(ShowPageInput::ShowGotten(show_result));
            }
            ShowPageInput::ShowGotten(show_result) => match show_result {
                Ok(show) => {
                    self.show = Some(show);
                    self.load_error = None;

                    if let Some(show) = &self.show {
                        if let Some(image_url_ref) = show.image_uri() {
                            let image_url = image_url_ref.to_string();
                            sender.oneshot_command(async move {
                                let texture_res = tokio::task::spawn_blocking(move || {
                                    let load_image = || -> Option<gtk::gdk::Texture> {
                                        let file = gtk::gio::File::for_uri(&image_url);
                                        let (glib_bytes, _) =
                                            file.load_bytes(gtk::gio::Cancellable::NONE).ok()?;

                                        const IMAGE_BANNER_SIZE: i32 = 500;
                                        let stream =
                                            gtk::gio::MemoryInputStream::from_bytes(&glib_bytes);
                                        let pixbuf = gtk::gdk_pixbuf::Pixbuf::from_stream_at_scale(
                                            &stream,
                                            IMAGE_BANNER_SIZE,
                                            IMAGE_BANNER_SIZE,
                                            true,
                                            gtk::gio::Cancellable::NONE,
                                        )
                                        .ok()?;

                                        Some(gtk::gdk::Texture::for_pixbuf(&pixbuf))
                                    };

                                    load_image()
                                })
                                .await;

                                let downloaded_texture = match texture_res {
                                    Ok(Some(texture)) => Some(texture),
                                    _ => None,
                                };

                                ShowPageCmdInput::DownloadImage(downloaded_texture)
                            });
                        }
                    }

                    if let Some(show) = &self.show {
                        match dbqueries::get_pd_episodes(show) {
                            Ok(episodes) => {
                                println!("Episodes Loaded: {:?}", episodes.len());
                                let mut guard = self.episodes.guard();
                                guard.clear();
                                for episode in episodes {
                                    guard.push_back(episode);
                                }
                            }
                            Err(error) => {
                                eprintln!("Error Loading Episodes: {}", error);
                            }
                        }
                    }
                }
                Err(error) => {
                    // NEW: store the error so the UI banner above actually
                    // shows it, instead of only this eprintln.
                    eprintln!("Error Loading Show Metadata: {}", error);
                    self.load_error = Some(format!("Failed to load show: {error}"));
                }
            },
            ShowPageInput::ImageDownloaded(opt_texture) => {
                self.show_image_texture = opt_texture;
            }
            ShowPageInput::FetchEpisodesComplete(data) => match data {
                Ok(episodes) => {
                    let mut guard = self.episodes.guard();
                    guard.clear();
                    for episode in episodes {
                        guard.push_back(episode);
                    }
                }
                Err(error) => {
                    eprintln!("Episodes sync failed: {}", error);
                }
            },
            _ => {}
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            ShowPageCmdInput::DownloadImage(opt_texture) => {
                sender.input(ShowPageInput::ImageDownloaded(opt_texture));
            }
        }
    }
}
