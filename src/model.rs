use std::path::PathBuf;

use crate::audio::detect::AudioFormat;
use crate::audio::probe::ProbeInfo;
use crate::audio::recommend::Recommendation;

/// Per-file pipeline state, from "just picked" to "done" or "failed".
#[derive(Debug, Clone, PartialEq)]
pub enum ItemStatus {
    Queued,
    Converting,
    Done(PathBuf),
    Failed(String),
}

/// One row in the batch queue.
#[derive(Debug, Clone)]
pub struct QueueItem {
    pub path: PathBuf,
    pub format: Option<AudioFormat>,
    pub detection_note: String,
    pub probe_info: Option<ProbeInfo>,
    pub recommendation: Option<Recommendation>,
    pub status: ItemStatus,
}

impl QueueItem {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            format: None,
            detection_note: "pending".to_string(),
            probe_info: None,
            recommendation: None,
            status: ItemStatus::Queued,
        }
    }
}

/// Messages the GUI can send to the app.
#[derive(Debug, Clone)]
pub enum AppMessage {
    BrowseFiles,
    AnalyzeAll,
    ConvertAll,
    RemoveItem(usize),
    ClearFinished,
}
