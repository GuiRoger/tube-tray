//! Lista os vídeos de uma playlist com `yt-dlp --flat-playlist -J`
//! (só metadados, sem abrir cada vídeo: alguns segundos mesmo para
//! playlists grandes).

use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::metadata::thumbnail_url;
use crate::runner::hidden_command;
use crate::youtube_url;

/// Máximo de vídeos listados de uma playlist.
pub const PLAYLIST_LIMIT: usize = 500;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistEntry {
    pub id: String,
    pub url: String,
    pub title: String,
    /// segundos
    pub duration: Option<f64>,
    pub thumbnail: String,
    /// vídeo privado ou removido (aparece desmarcado e desabilitado)
    pub available: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistInfo {
    pub id: String,
    pub title: String,
    pub channel: String,
    pub entries: Vec<PlaylistEntry>,
    /// a playlist tem mais vídeos que [`PLAYLIST_LIMIT`]
    pub truncated: bool,
}

#[derive(Deserialize)]
struct RawPlaylist {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    channel: Option<String>,
    #[serde(default)]
    uploader: Option<String>,
    #[serde(default)]
    playlist_count: Option<u64>,
    #[serde(default)]
    entries: Vec<Option<RawEntry>>,
}

#[derive(Deserialize)]
struct RawEntry {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    duration: Option<f64>,
    #[serde(default)]
    availability: Option<String>,
}

/// Interpreta o JSON do `yt-dlp --flat-playlist -J`.
pub fn parse(json: &str) -> Result<PlaylistInfo, String> {
    let raw: RawPlaylist =
        serde_json::from_str(json).map_err(|_| "Resposta inesperada do yt-dlp.".to_string())?;
    let mut seen = std::collections::HashSet::new();
    let entries: Vec<PlaylistEntry> = raw
        .entries
        .into_iter()
        .flatten()
        .filter_map(|e| {
            let id = e.id?;
            let url = youtube_url::normalize(&format!("https://youtu.be/{id}"))?;
            if !seen.insert(id.clone()) {
                return None;
            }
            let title = e.title.unwrap_or_default();
            let hidden = matches!(title.as_str(), "[Private video]" | "[Deleted video]")
                || matches!(e.availability.as_deref(), Some("private" | "needs_auth" | "subscriber_only"));
            Some(PlaylistEntry {
                thumbnail: thumbnail_url(&id),
                title: if title.is_empty() { id.clone() } else { title },
                id,
                url,
                duration: e.duration,
                available: !hidden,
            })
        })
        .take(PLAYLIST_LIMIT)
        .collect();
    if entries.is_empty() {
        return Err("Essa playlist está vazia ou é privada.".into());
    }
    let truncated = raw.playlist_count.is_some_and(|c| c as usize > entries.len()) && entries.len() >= PLAYLIST_LIMIT;
    Ok(PlaylistInfo {
        id: raw.id.unwrap_or_default(),
        title: raw.title.filter(|t| !t.is_empty()).unwrap_or_else(|| "Playlist".into()),
        channel: raw.channel.or(raw.uploader).unwrap_or_default(),
        entries,
        truncated,
    })
}

/// Busca os vídeos da playlist do link (`list=`).
pub async fn playlist_info(ytdlp: &Path, input: &str) -> Result<PlaylistInfo, String> {
    let url = youtube_url::playlist_url(input).ok_or("Esse link não é de uma playlist.")?;
    let end = PLAYLIST_LIMIT.to_string();
    let fut = hidden_command(ytdlp)
        .args([
            "--flat-playlist",
            "--yes-playlist",
            "-J",
            "--no-warnings",
            "--encoding",
            "utf-8",
            "--playlist-end",
            &end,
            "--",
            &url,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .output();
    let out = tokio::time::timeout(Duration::from_secs(60), fut)
        .await
        .map_err(|_| "A playlist demorou demais para responder.".to_string())?
        .map_err(|e| format!("Não foi possível executar o yt-dlp: {e}"))?;
    if !out.status.success() {
        let err = crate::files::decode_output(&out.stderr);
        let line = err.lines().rev().find(|l| l.starts_with("ERROR:")).unwrap_or("");
        return Err(if line.is_empty() {
            "Não foi possível abrir a playlist.".into()
        } else {
            crate::progress::friendly_error(line.trim_start_matches("ERROR:").trim())
        });
    }
    parse(&crate::files::decode_output(&out.stdout))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_flat_playlist() {
        let json = r#"{
            "id": "PL123", "title": "Aulas de Rust", "uploader": "Canal",
            "playlist_count": 4,
            "entries": [
                {"id": "aaaaaaaaaaa", "title": "Aula 1", "duration": 61.0},
                {"id": "bbbbbbbbbbb", "title": "[Private video]", "duration": null},
                {"id": "aaaaaaaaaaa", "title": "Aula 1 (repetida)"},
                null,
                {"id": "curto", "title": "inválido"},
                {"id": "ccccccccccc", "title": "", "availability": "public"}
            ]
        }"#;
        let p = parse(json).unwrap();
        assert_eq!(p.title, "Aulas de Rust");
        assert_eq!(p.channel, "Canal");
        assert_eq!(p.entries.len(), 3);
        assert_eq!(p.entries[0].url, "https://www.youtube.com/watch?v=aaaaaaaaaaa");
        assert_eq!(p.entries[0].duration, Some(61.0));
        assert!(!p.entries[1].available);
        assert_eq!(p.entries[2].title, "ccccccccccc");
        assert!(!p.truncated);
    }

    #[test]
    fn empty_playlist_is_an_error() {
        assert!(parse(r#"{"id":"PL1","entries":[]}"#).is_err());
        assert!(parse("lixo").is_err());
    }
}
