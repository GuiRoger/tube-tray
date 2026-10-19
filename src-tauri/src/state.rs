use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tubetray_core::{DownloadManager, Settings};

pub struct AppState {
    pub manager: DownloadManager,
    pub settings: Arc<Mutex<Settings>>,
    pub data_dir: PathBuf,
}

impl AppState {
    pub fn ytdlp_path(&self) -> PathBuf {
        tubetray_core::tools::ytdlp_path(&self.data_dir.join("bin"))
    }
}
