use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex, RwLock};
use std::time::Duration;

use log::debug;
use podcasts_data::EpisodeWidgetModel;
use podcasts_data::downloader::get_episode;
use podcasts_data::errors::DownloadError;
use relm4::ComponentSender;
use tokio::time::interval;

use podcasts_data::{EpisodeId, dbqueries, downloader::DownloadProgress, utils::get_download_dir};

use crate::workers::action_worker::service::{ActionWorker, ActionWorkerOutput};

// ---------------------------------------------------------------------
// Progress tracking — identical shape/logic to the original manager.rs,
// including the debug! calls in get_fraction.
// ---------------------------------------------------------------------

pub(crate) type ActiveProgress = Arc<Mutex<Progress>>;
pub(crate) type DownloadProgressLock = Arc<RwLock<HashMap<EpisodeId, ActiveProgress>>>;

pub(crate) static ACTIVE_DOWNLOADS: LazyLock<DownloadProgressLock> =
    LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));

#[derive(Debug, Default)]
pub(crate) struct Progress {
    total_bytes: u64,
    downloaded_bytes: u64,
    cancel: bool,
}

impl Progress {
    pub(crate) fn get_fraction(&self) -> f64 {
        let ratio = self.downloaded_bytes as f64 / self.total_bytes as f64;
        println!("{:?}", self);
        println!("Ratio completed: {}", ratio);

        if ratio >= 1.0 {
            return 1.0;
        };
        ratio
    }
}

impl DownloadProgress for Progress {
    fn get_downloaded(&self) -> u64 {
        self.downloaded_bytes
    }

    fn set_downloaded(&mut self, downloaded: u64) {
        self.downloaded_bytes = downloaded
    }

    fn set_size(&mut self, bytes: u64) {
        self.total_bytes = bytes;
    }

    fn get_size(&self) -> u64 {
        self.total_bytes
    }

    fn should_cancel(&self) -> bool {
        self.cancel
    }

    fn cancel(&mut self) {
        self.cancel = true;
    }
}

impl ActionWorker {
    pub async fn download_podcast_episode(sender: ComponentSender<Self>, id: EpisodeId) {
        // `downloader::get_episode` takes `&mut EpisodeWidgetModel`, not
        // `&mut Episode` — fetch the widget model fresh from the DB here,
        // exactly like the original manager.rs did with
        // `dbqueries::get_episode_widget_from_id(id)`.
        let mut episode: EpisodeWidgetModel = match dbqueries::get_episode_widget_from_id(id) {
            Ok(ep) => ep,
            Err(e) => {
                sender
                    .output(ActionWorkerOutput::ErrorNotification(format!(
                        "Download failed: {e}"
                    )))
                    .ok();
                return;
            }
        };

        // Resolve the podcast + on-disk download directory for this episode.
        let download_dir = match dbqueries::get_podcast_from_id(episode.show_id()) {
            Ok(pd) => match get_download_dir(pd.title()) {
                Ok(dir) => dir,
                Err(e) => {
                    sender
                        .output(ActionWorkerOutput::ErrorNotification(format!(
                            "Download failed: {e}"
                        )))
                        .ok();
                    return;
                }
            },
            Err(e) => {
                sender
                    .output(ActionWorkerOutput::ErrorNotification(format!(
                        "Download failed: {e}"
                    )))
                    .ok();
                return;
            }
        };

        // Create a new `Progress` struct to keep track of dl progress and
        // register it so `CancelDownload` can find it later.
        let prog: ActiveProgress = Arc::new(Mutex::new(Progress::default()));
        match ACTIVE_DOWNLOADS.write() {
            Ok(mut guard) => {
                guard.insert(id, prog.clone());
            }
            Err(err) => {
                sender
                    .output(ActionWorkerOutput::ErrorNotification(format!(
                        "ActiveDownloads: {err}."
                    )))
                    .ok();
                return;
            }
        }

        sender.output(ActionWorkerOutput::DownloadStarted(id)).ok();

        // Poll the shared `Progress` struct concurrently with the download
        // itself and stream fraction updates back out through the same
        // `ComponentSender`, so the UI can drive a progress bar without
        // any separate channel.
        let poll_prog = prog.clone();
        let poll_sender = sender.clone();
        let progress_ticker = relm4::spawn(async move {
            let mut ticker = interval(Duration::from_millis(250));
            loop {
                ticker.tick().await;
                let fraction = match poll_prog.lock() {
                    Ok(p) => p.get_fraction(),
                    Err(_) => break,
                };
                if poll_sender
                    .output(ActionWorkerOutput::DownloadProgress { id, fraction })
                    .is_err()
                {
                    // Parent dropped its receiving end; stop polling.
                    break;
                }
                if fraction >= 1.0 {
                    break;
                }
            }
        });

        match get_episode(&mut episode, download_dir.as_str(), Some(prog)).await {
            Ok(_) => {
                sender.output(ActionWorkerOutput::DownloadFinished(id)).ok();
            }
            Err(DownloadError::DownloadCancelled) => {
                sender
                    .output(ActionWorkerOutput::DownloadCancelled(id))
                    .ok();
            }
            Err(e) => {
                sender
                    .output(ActionWorkerOutput::ErrorNotification(format!(
                        "Download failed: {e}"
                    )))
                    .ok();
            }
        }

        // Stop the progress ticker now that the download has settled, then
        // clean up the registry entry — same as the original manager.rs.
        progress_ticker.abort();

        if let Ok(mut m) = ACTIVE_DOWNLOADS.write() {
            let progress = m.remove(&id);
            debug!("Removed: {:?}", progress);
        }

        sender.output(ActionWorkerOutput::RefreshEpisode(id)).ok();
    }

    /// Ask an in-flight download to cancel. No-op (with a notification) if
    /// nothing is currently downloading for this episode.
    pub fn cancel_download(id: EpisodeId, sender: &ComponentSender<Self>) {
        let guard = match ACTIVE_DOWNLOADS.read() {
            Ok(g) => g,
            Err(_) => return,
        };

        if let Some(prog) = guard.get(&id) {
            if let Ok(mut p) = prog.lock() {
                p.cancel();
                return;
            }
        }

        sender
            .output(ActionWorkerOutput::ErrorNotification(
                "No active download to cancel".into(),
            ))
            .ok();
    }
}
