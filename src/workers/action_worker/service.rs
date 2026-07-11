use std::sync::Arc;

use gst::glib::object::ObjectExt;
use gst_play::PlayState;
use gtk::gio::prelude::FileExt;
use gst::prelude::*;
use mpris_player::{Metadata, MprisPlayer, PlaybackStatus};
use podcasts_data::Episode;
use podcasts_data::EpisodeModel;
use podcasts_data::FEED_MANAGER;
use podcasts_data::nextcloud_sync;
use podcasts_data::nextcloud_sync::SyncError;
use podcasts_data::nextcloud_sync::SyncPolicy;
use podcasts_data::nextcloud_sync::SyncResult;
use relm4::{ComponentSender, Worker};

use podcasts_data::EpisodeId;
use podcasts_data::dbqueries; // adjust import path/name if yours differs

use crate::action::Action;
use crate::util::gst_errors::handel_gst_core_error;
use crate::util::gst_errors::handel_gst_resource_error;
use crate::util::gst_errors::handel_gst_stream_error;
use crate::workers::action_worker::service::ActionWorkerInput::Execute;

// -----------------------------------------------------------------------
// NOTE ON TYPES: this file assumes `Action` (and every payload it carries —
// Podcast, Show, Episode/EpisodeWidgetModel, FoundPodcast, Chapters, etc.)
// already derives `Clone` + `Debug`, since it now travels across a channel
// to a background thread and gets broadcast to possibly many subscribers.
// If any of those types currently don't derive Clone, that's the first
// compile error you'll hit — add `#[derive(Clone)]` to them (they're likely
// already cheap: Arc<Podcast>, ids, small structs).
// -----------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum ActionWorkerInput {
    Subscirbe(String),
    /// Run an action. This takes the exact same `Action` enum you already
    /// have — no call sites need to change, they just send here instead of
    /// calling `self.do_action(...)` directly.
    Execute(Action),
    /// Sent by the worker's own spawned tasks back into itself, to flip
    /// internal flags (like "sync in progress") from the worker's own
    /// thread instead of racing with it from inside a spawned future.
    SyncFinished,
    TogglePlayBack,
    StateChanged(gst_play::PlayState),
}

#[derive(Debug, Clone)]
pub enum ActionWorkerOutput {
    Forward(Action),
    SetUpdatingState(bool),
    EpisodeReady(podcasts_data::EpisodeWidgetModel), // swap for your real type
    UriReady(String),
    NotifyError(String),
    SyncFinished,
    StateChanged(gst_play::PlayState),
    PositionChanged(u64),
    SetCurrentEpisode(EpisodeId),
    RefreshAllViews
}

#[derive(Debug, Clone)]
pub enum ActionWorkerCmd {}

// --- Thread Communication Commands ---
#[derive(Debug)]
pub enum MprisCommand {
    ChangePlaybackState(gst_play::PlayState),
    UpdateMetadata {
        title: String,
        show_title: String,
        art_url: Option<String>,
    },
}


pub struct ActionWorker {
    syncing: bool,
    player: gst_play::Play,
    _player_signals: gst_play::PlaySignalAdapter,
    current_player_state: gst_play::PlayState,
    current_episode: Option<EpisodeId>,
     mpris_tx: async_channel::Sender<MprisCommand>,
}

impl Worker for ActionWorker {
    type Init = ();
    type Input = ActionWorkerInput;
    type Output = ActionWorkerOutput;

    fn init(_init: Self::Init, sender: ComponentSender<Self>) -> Self {
        let player = gst_play::Play::default();
        let mut config = player.config();

        const USER_AGENT: &str = "XPodcasts/1.0";
        config.set_user_agent(USER_AGENT);
        config.set_position_update_interval(250); 
        player.set_config(config).unwrap();
        player.set_video_track_enabled(false);

        let player_signals = gst_play::PlaySignalAdapter::new(&player);

        let position_sender = sender.clone();
        player_signals.connect_duration_changed(move |_, position| {
            if let Some(pos) = position {
                let _ = position_sender.output(ActionWorkerOutput::PositionChanged(pos.mseconds()));
            }
        });

        let error_sender = sender.clone();
        player_signals.connect_error(move |_player, error, _details| {
            let raw_error_msg = error.to_string();
            let error_msg;

            // 1. First try matching strict type variants
            if let Some(res_err) = error.kind::<gst::ResourceError>() {
                error_msg = handel_gst_resource_error(res_err);
            } else if let Some(stream_err) = error.kind::<gst::StreamError>() {
                error_msg = handel_gst_stream_error(stream_err);
            } else if let Some(core_err) = error.kind::<gst::CoreError>() {
                error_msg = handel_gst_core_error(core_err);
            // 2. Smart String Fallback to clean up unmapped low-level strings
            } else if raw_error_msg.contains("souphttpsrc") || raw_error_msg.contains("reason error (-5)") {
                error_msg = "Could not stream the podcast due to a network connection timeout or a bad server response.".to_string();
            } else {
                error_msg = "An unexpected playback error occurred.".to_string();
            }

            let _ = error_sender.output(ActionWorkerOutput::NotifyError(error_msg));
        });


        let state_sender = sender.clone();
        player_signals.connect_state_changed(move |_, state| {
            let _ = state_sender.input(ActionWorkerInput::StateChanged(state));
        });

       let (mpris_tx, mpris_rx) = async_channel::unbounded::<MprisCommand>();
        let loopback_sender = sender.clone();

        relm4::gtk::glib::MainContext::default().spawn_local(async move {
            struct GlobalMpris {
           player: std::sync::Arc<mpris_player::MprisPlayer>,
        }
            
            thread_local! {
                static INSTANCE: std::cell::RefCell<Option<GlobalMpris>> = const { std::cell::RefCell::new(None) };
            }


              while let Ok(cmd) = mpris_rx.recv().await {
                INSTANCE.with(|cell| {
                    let mut cell = cell.borrow_mut();
                   let state = cell.get_or_insert_with(|| {
                        let p = mpris_player::MprisPlayer::new(
                            "org.mpris.MediaPlayer2.ZoePodcastApp".to_string(),
                            "XPodcasts".to_string(),
                            "".to_string(),
                        );
                        
                        let inner_sender = loopback_sender.clone();
                        p.connect_play_pause(move || {
                            let _ = inner_sender.input(ActionWorkerInput::TogglePlayBack);
                        });

                        // FIX: Wrap the Arc inside the GlobalMpris struct container
                        GlobalMpris { player: p }
                    });
                    let player_ref = &state.player;

             match cmd {
                        MprisCommand::ChangePlaybackState(state) => {
                            let status = match state {
                                gst_play::PlayState::Playing => mpris_player::PlaybackStatus::Playing,
                                gst_play::PlayState::Paused => mpris_player::PlaybackStatus::Paused,
                                _ => mpris_player::PlaybackStatus::Stopped,
                            };
                            player_ref.set_playback_status(status);
                        }
                        MprisCommand::UpdateMetadata { title, show_title, art_url } => {
                            let mut metadata = mpris_player::Metadata::new();
                            metadata.title = Some(title);
                            metadata.artist = Some(vec![show_title]);

                            if let Some(ref remote_url) = art_url {
                                if !remote_url.is_empty() {
                                    let mut cache_path = adw::glib::user_cache_dir();
                                    cache_path.push("xpodcasts");
                                    cache_path.push("covers");

                                    let glib_url_bytes = adw::glib::Bytes::from(remote_url.as_bytes());
                                    if let Some(hashed_name) = adw::glib::compute_checksum_for_bytes(
                                        adw::glib::ChecksumType::Sha256,
                                        &glib_url_bytes
                                    ) {
                                        let local_disk_file = cache_path.join(hashed_name.as_str());

                                        if local_disk_file.exists() {
                                        let uri_string = adw::gio::File::for_path(local_disk_file).uri();
                                            metadata.art_url = Some(uri_string.to_string());
                                        } else {
                                            metadata.art_url = Some(remote_url.clone());
                                        }
                                    }
                                }
                            }

                            player_ref.set_metadata(metadata);
                        }

                    }
                });
            }
        });

       Self {
            syncing: false,
            player,
            current_player_state: gst_play::PlayState::Stopped,
            _player_signals: player_signals,
            current_episode: None,
            mpris_tx, 
        }
    }

    fn update(&mut self, input: Self::Input, sender: ComponentSender<Self>) {
        match input {
            ActionWorkerInput::Execute(action) => self.execute(action, sender.clone()),
            ActionWorkerInput::SyncFinished => {
                self.syncing = false;
            }
            ActionWorkerInput::Subscirbe(feed) => {
             relm4::tokio::spawn(async move {
                    Self::subscribe(sender, feed).await;
                });
            }
            ActionWorkerInput::StateChanged(state) => {
                self.current_player_state = state;

              let _ = self.mpris_tx.send_blocking(MprisCommand::ChangePlaybackState(state));
                let _ = sender.output(ActionWorkerOutput::StateChanged(state));
            }
            ActionWorkerInput::TogglePlayBack => match self.current_player_state {
                gst_play::PlayState::Stopped | gst_play::PlayState::Paused => {
                    self.player.play();
                }
                gst_play::PlayState::Playing => {
                    self.player.pause();
                }
                _ => {}
            },
        }
    }
}

impl ActionWorker {
    fn execute(&mut self, action: Action, sender: ComponentSender<Self>) {
        match action {
            // ----------------------------------------------------------
            // Background-work actions: the worker does something real
            // here (DB read/write, network call) before broadcasting.
            // ----------------------------------------------------------
            Action::RefreshEpisode(id) => self.refresh_episode(id, sender.clone()),
            
            Action::RefreshAllViews => {
                 let _ = sender.output(ActionWorkerOutput::RefreshAllViews);
            }

            Action::RefreshWidgetIfSame(id) => {
                let _ = sender.output(ActionWorkerOutput::Forward(Action::RefreshWidgetIfSame(id)));
            }

            Action::MarkAsPlayed(played, id) => self.mark_as_played(played, id, sender),

            Action::CopyUrl(id) => self.copy_url(id, sender.clone()),

            Action::QuickSyncNextcloud => self.quick_sync_nextcloud(sender),

            Action::FeedRefreshed(id) => {
                let _ = sender.output(ActionWorkerOutput::Forward(Action::FeedRefreshed(id)));
            }
            Action::StreamEpisode(id) => {
                self.current_episode = Some(id);

                if let Some(id) = self.current_episode {
                    let _ = sender.output(ActionWorkerOutput::SetCurrentEpisode(id));
                    match dbqueries::get_episode_from_id(id) {
                        Ok(episode) => {
                            if let Some(stream_url) = episode.uri() {
                                self.player.set_uri(Some(&stream_url));
                                self.player.play();
                            }

                            let title = episode.title().to_string();
                            let mut show_title = "Unknown Podcast".to_string();

                            if let Ok(show) =
                                dbqueries::get_podcast_cover_from_id(episode.show_id())
                            {
                                show_title = show.title().to_string();
                            }

                            let art_url = episode.image_uri().map(|uri| uri.to_string());

                            // Dispatch metadata down the channel line
                            let _ = self.mpris_tx.try_send(MprisCommand::UpdateMetadata {
                                title,
                                show_title,
                                art_url,
                            });
                        }
                        Err(error) => {
                            let _ =
                                sender.output(ActionWorkerOutput::NotifyError(error.to_string()));
                        }
                    };
                };
            }
            Action::Pause => {
                self.player.pause();
            }
            Action::Play => {
                self.player.play();
            }

            // ----------------------------------------------------------
            // Pure UI actions: no background work exists to do. These are
            // navigation, toasts, window chrome, or app-level state
            // (inhibit/uninhibit) that only make sense on the main thread
            // where `window`/`self: &Application` actually live. Forward
            // them unchanged so the subscriber runs the exact same widget
            // code your original `do_action` had.
            // ----------------------------------------------------------
            other @ (Action::RefreshShowsView
            | Action::RefreshEpisodesView
            | Action::ReplaceWidget(_)
            | Action::GoToEpisodeDescription(_, _)
            | Action::GoToShow(_)
            | Action::GoToFoundPodcasts(_)
            | Action::GoToChaptersPage(_, _)
            | Action::ChaptersAvailable(_, _)
            | Action::CopiedUrlNotification
            | Action::MarkAllPlayerNotification(_)
            | Action::RemoveShow(_)
            | Action::ErrorNotification(_)
            | Action::StartUpdating
            | Action::StopUpdating
            | Action::InitEpisode(_)
            | Action::InitEpisodeAt(_, _)
            | Action::UpdateCover(_)
            | Action::EmptyState
            | Action::PopulatedState
            | Action::RaiseWindow
            | Action::InhibitSuspend
            | Action::UninhibitSuspend) => {
                let _ = sender.output(ActionWorkerOutput::Forward(other));
            }
        }
    }

    // -------------------------------------------------------------
    // Background-work implementations
    // -------------------------------------------------------------

    fn refresh_episode(&self, id: EpisodeId, sender: ComponentSender<Self>) {
        match dbqueries::get_episode_widget_from_id(id) {
            Ok(ep) => {
                let _ = sender.output(ActionWorkerOutput::EpisodeReady(ep));
            }
            Err(e) => {
                let _ = sender.output(ActionWorkerOutput::NotifyError(format!(
                    "failed to fetch episode for description refresh: {e}"
                )));
            }
        }
    }

    fn mark_as_played(&self, played: bool, id: EpisodeId, sender: ComponentSender<Self>) {
        let mut ep = match dbqueries::get_episode_widget_from_id(id) {
            Ok(ep) => ep,
            Err(e) => {
                let _ = sender.output(ActionWorkerOutput::NotifyError(format!(
                    "failed to fetch episode to mark played: {e}"
                )));
                return;
            }
        };

        let result = if played {
            ep.set_played_now()
        } else {
            ep.set_unplayed()
        };
        if let Err(e) = result {
            let _ = sender.output(ActionWorkerOutput::NotifyError(format!(
                "failed to update played state: {e}"
            )));
            return;
        }

        let _ = sender.output(ActionWorkerOutput::EpisodeReady(ep.clone()));

        sender.input(ActionWorkerInput::Execute(Action::QuickSyncNextcloud));
        sender.input(ActionWorkerInput::Execute(Action::RefreshEpisode(ep.id())));
    }

    fn copy_url(&self, id: EpisodeId, sender: ComponentSender<Self>) {
        match dbqueries::get_episode_from_id(id)
            .ok()
            .and_then(|e| e.local_uri().map(|s| s.to_string()))
        {
            Some(uri) => {
                let _ = sender.output(ActionWorkerOutput::UriReady(uri));
            }
            None => {
                let _ = sender.output(ActionWorkerOutput::NotifyError(
                    "no URL available for that episode".to_string(),
                ));
            }
        }
    }

    fn quick_sync_nextcloud(&mut self, sender: ComponentSender<Self>) {
        if self.syncing {
            // mirrors the old `window.updating()` guard
            return;
        }
        self.syncing = true;

        let _ = sender.output(ActionWorkerOutput::SetUpdatingState(true));

        crate::RUNTIME.spawn(async move {
            let result = nextcloud_sync::sync(SyncPolicy::CancelOnMissingEpisodes).await;

            match result {
                Ok(SyncResult::Done {
                    episode_updates_downloaded,
                    subscription_updates_downloaded,
                }) => {
                    if episode_updates_downloaded > 0 || subscription_updates_downloaded > 0 {
                        let _ = sender.output(ActionWorkerOutput::Forward(Action::RefreshAllViews));
                    }
                }
                Ok(SyncResult::Skipped) => {}
                Err(SyncError::DownloadedUpdateForEpisodeNotInDb) => {
                    let errors = FEED_MANAGER.full_refresh().await;
                    let errors = FEED_MANAGER.retry_errors_full(errors).await;
                    let _ = FEED_MANAGER.retry_errors_full(errors).await;

                    match nextcloud_sync::sync(SyncPolicy::IgnoreMissingEpisodes).await {
                        Ok(_) => {
                            let _ =
                                sender.output(ActionWorkerOutput::Forward(Action::RefreshAllViews));
                        }
                        Err(e) => {
                            let _ = sender.output(ActionWorkerOutput::NotifyError(format!(
                                "Sync failed {e}"
                            )));
                        }
                    }
                }
                Err(e) => {
                    let _ =
                        sender.output(ActionWorkerOutput::NotifyError(format!("Sync failed {e}")));
                }
            }

            let _ = sender.output(ActionWorkerOutput::SetUpdatingState(false));

            // flip `syncing` back off on the worker's own thread instead
            // of racing with it from here
            sender.input(ActionWorkerInput::SyncFinished);
        });
    }

    // The mpris-player crate allows you to process incoming hardware keys
    // like this inside your event loop or an async task thread:
    fn handle_hardware_keys(&self) {
        let player_handle = self.player.clone();

        // Spawn a listener thread to catch OS/Hardware playback requests
        std::thread::spawn(move || {
            // Blocks waiting for system D-Bus commands from headphone hooks
            // event_loop.run_once() or similar depending on chosen executor
        });
    }
}
