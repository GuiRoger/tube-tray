//! TubeTray core — tudo que não depende de UI.
//!
//! - [`youtube_url`]: valida e normaliza links
//! - [`profile`]: argumentos de formato (MP4, melhor qualidade, M4A, MP3)
//! - [`playlist`]: vídeos de uma playlist (`--flat-playlist`)
//! - [`progress`]: protocolo de linhas `TT_*` e agregação de progresso
//! - [`runner`]: executa um download do yt-dlp
//! - [`queue`]: fila com downloads simultâneos, cancelamento e retry
//! - [`tools`]: baixa/atualiza o yt-dlp e localiza o ffmpeg
//! - [`settings`]: configurações em JSON
//! - [`files`]: decodificação da saída e localização do arquivo final
//! - [`metadata`]: prévia do vídeo (título, canal, miniatura)

pub mod files;
pub mod metadata;
pub mod playlist;
pub mod profile;
pub mod progress;
pub mod queue;
pub mod runner;
pub mod settings;
pub mod tools;
pub mod youtube_url;

pub use metadata::VideoInfo;
pub use playlist::{PlaylistEntry, PlaylistInfo};
pub use profile::{MediaKind, Profile};
pub use progress::{Snapshot, Stage};
pub use queue::{DownloadManager, EnqueueError, Job, RunEnv};
pub use settings::Settings;
