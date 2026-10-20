use std::path::PathBuf;

use iced::{Element, Task, Theme};

use crate::audio::{convert, detect, probe, recommend};
use crate::config::AppConfig;
use crate::gui;
use crate::model::{AppMessage, ItemStatus, QueueItem};

/// Top-level application state, owned by the iced runtime.
#[derive(Debug)]
pub struct RenosancseApp {
    pub config: AppConfig,
    pub items: Vec<QueueItem>,
    pub status: String,
    pub ffmpeg_available: bool,
}

impl RenosancseApp {
    pub fn new() -> (Self, Task<AppMessage>) {
        let config = AppConfig::load();
        config.save();
        let ffmpeg_available = which::which("ffmpeg").is_ok();
        let status = if ffmpeg_available {
            "Ready. Add audio files to get started.".to_string()
        } else {
            "Ready. FFmpeg was not found on PATH — install it to enable conversion.".to_string()
        };
        let app = Self {
            config,
            items: Vec::new(),
            status,
            ffmpeg_available,
        };
        (app, Task::none())
    }

    pub fn theme(_app: &Self) -> Theme {
        Theme::TokyoNight
    }

    pub fn view(&self) -> Element<'_, AppMessage> {
        gui::render(self)
    }

    pub fn update(app: &mut Self, msg: AppMessage) -> Task<AppMessage> {
        match msg {
            AppMessage::BrowseFiles => app.browse(),
            AppMessage::AnalyzeAll => app.analyze(),
            AppMessage::ConvertAll => app.convert_all(),
            AppMessage::RemoveItem(index) => {
                if index < app.items.len() {
                    app.items.remove(index);
                }
                app.status = "Removed file.".to_string();
            }
            AppMessage::ClearFinished => {
                app.items
                    .retain(|item| !matches!(item.status, ItemStatus::Done(_)));
                app.status = "Cleared finished items.".to_string();
            }
        }
        Task::none()
    }
}

impl RenosancseApp {
    /// Open the native file picker and add chosen files to the queue.
    fn browse(&mut self) {
        let picked: Vec<PathBuf> = rfd::FileDialog::new()
            .set_title("Choose audio files")
            .pick_files()
            .unwrap_or_default();
        let mut added = 0usize;
        for path in picked {
            if self.items.iter().any(|item| item.path == path) {
                continue;
            }
            let mut item = QueueItem::new(path.clone());
            match detect::detect(&path) {
                Ok(detection) => {
                    log::debug!("detected {}: {}", path.display(), detection.summary());
                    item.format = Some(detection.format);
                    item.detection_note = detection.summary();
                }
                Err(err) => {
                    item.detection_note = format!("unreadable: {err}");
                    item.status = ItemStatus::Failed(err.to_string());
                }
            }
            self.items.push(item);
            added += 1;
        }
        if added > 0 {
            self.status = format!("Added {added} file(s).");
        }
    }

    /// Probe every queued file with ffprobe (when available) and compute a
    /// recommendation. v0.1 runs this synchronously — see the README roadmap.
    fn analyze(&mut self) {
        for item in &mut self.items {
            let Some(format) = item.format else { continue };
            let probed = probe::probe(&item.path).ok();
            item.probe_info = probed.clone();
            item.recommendation = Some(recommend::recommend(format, probed.as_ref()));
        }
        self.status = format!("Analyzed {} file(s).", self.items.len());
    }

    /// Convert every analyzed file. Output goes to the configured output
    /// directory, or next to the input when none is set.
    fn convert_all(&mut self) {
        let output_dir = self.config.output_dir.clone();
        let mut converted = 0usize;
        let mut failed = 0usize;
        for item in &mut self.items {
            let Some(rec) = item.recommendation.clone() else {
                continue;
            };
            item.status = ItemStatus::Converting;

            let name = item
                .path
                .file_stem()
                .map(|stem| format!("{}.{}", stem.to_string_lossy(), rec.output_ext))
                .unwrap_or_else(|| format!("renosancse.{}", rec.output_ext));
            let output = match &output_dir {
                Some(dir) => dir.join(&name),
                None => item.path.with_file_name(&name),
            };

            let job = convert::ConvertJob {
                input: item.path.clone(),
                output,
                codec_args: rec.codec_args,
            };
            log::info!(
                "converting {} -> {}",
                job.input.display(),
                job.output.display()
            );
            match convert::convert(&job) {
                Ok(path) => {
                    item.status = ItemStatus::Done(path);
                    converted += 1;
                }
                Err(err) => {
                    item.status = ItemStatus::Failed(err.to_string());
                    failed += 1;
                }
            }
        }
        self.status = format!("Converted {converted} file(s), {failed} failed.");
    }
}
