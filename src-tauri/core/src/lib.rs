//! TubeTray core — tudo que não depende de UI.
//!
//! - [`youtube_url`]: valida e normaliza links
//! - [`profile`]: argumentos de formato (perfil MP4 atual)
//! - [`progress`]: protocolo de linhas `TT_*` e agregação de progresso
//! - [`runner`]: executa um download do yt-dlp
//! - [`queue`]: fila com um worker, cancelamento e retry
//! - [`tools`]: baixa/atualiza o yt-dlp e localiza o ffmpeg
//! - [`settings`]: configurações em JSON
//! - [`metadata`]: prévia do vídeo (título, canal, miniatura)

pub mod metadata;
pub mod profile;
pub mod progress;
pub mod queue;
pub mod runner;
pub mod settings;
pub mod tools;
pub mod youtube_url;

pub use metadata::VideoInfo;
pub use profile::Profile;
pub use progress::{Snapshot, Stage};
pub use queue::{DownloadManager, EnqueueError, Job, RunEnv};
pub use settings::Settings;
