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
}

#[derive(Debug, Clone)]
pub enum ActionWorkerOutput {
    Forward(Action),
    SetUpdatingState(bool),
    EpisodeReady(podcasts_data::EpisodeWidgetModel), // swap for your real type
    UriReady(String),
    NotifyError(String),
    SyncFinished,
}

#[derive(Debug, Clone)]
pub enum ActionWorkerCmd {}

pub struct ActionWorker {
    syncing: bool,
}

impl Worker for ActionWorker {
    type Init = ();
    type Input = ActionWorkerInput;
    type Output = ActionWorkerOutput;

    fn init(_init: Self::Init, _sender: ComponentSender<Self>) -> Self {
        Self { syncing: false }
    }

    fn update(&mut self, input: Self::Input, sender: ComponentSender<Self>) {
        match input {
            ActionWorkerInput::Execute(action) => self.execute(action, sender),
            ActionWorkerInput::SyncFinished => {}
            ActionWorkerInput::Subscirbe(feed) => {
                relm4::tokio::spawn(async move {
                    println!("subscribing");
                    Self::subscribe(sender, feed).await;
                });
            }
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

            Action::RefreshWidgetIfSame(id) => {
                let _ = sender.output(ActionWorkerOutput::Forward(Action::RefreshWidgetIfSame(id)));
            }

            Action::MarkAsPlayed(played, id) => self.mark_as_played(played, id, sender),

            Action::CopyUrl(id) => self.copy_url(id, sender.clone()),

            Action::QuickSyncNextcloud => self.quick_sync_nextcloud(sender),

            Action::FeedRefreshed(id) => {
                let _ = sender.output(ActionWorkerOutput::Forward(Action::FeedRefreshed(id)));
            }

            // ----------------------------------------------------------
            // Pure UI actions: no background work exists to do. These are
            // navigation, toasts, window chrome, or app-level state
            // (inhibit/uninhibit) that only make sense on the main thread
            // where `window`/`self: &Application` actually live. Forward
            // them unchanged so the subscriber runs the exact same widget
            // code your original `do_action` had.
            // ----------------------------------------------------------
            other @ (Action::RefreshAllViews
            | Action::RefreshShowsView
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
            | Action::StreamEpisode(_)
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
}
