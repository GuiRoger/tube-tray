//! Prévia do vídeo (título, canal e miniatura) antes de baixar.
//!
//! Usa o oEmbed público do YouTube: uma requisição pequena, sem chave de
//! API e bem mais rápida que `yt-dlp -J`.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::youtube_url;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VideoInfo {
    pub id: String,
    pub url: String,
    pub title: String,
    pub channel: String,
    /// miniatura 320×180 (16:9, sem tarjas)
    pub thumbnail: String,
}

#[derive(Deserialize)]
struct OEmbed {
    title: String,
    #[serde(default)]
    author_name: String,
}

pub fn thumbnail_url(id: &str) -> String {
    format!("https://i.ytimg.com/vi/{id}/mqdefault.jpg")
}

/// Busca título e canal. Erros já vêm em português, prontos para a UI.
pub async fn video_info(input: &str) -> Result<VideoInfo, String> {
    let id = youtube_url::video_id(input).ok_or("Esse link não parece ser do YouTube.")?;
    let url = format!("https://www.youtube.com/watch?v={id}");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .user_agent(concat!("TubeTray/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .get("https://www.youtube.com/oembed")
        .query(&[("url", url.as_str()), ("format", "json")])
        .send()
        .await
        .map_err(|_| "Sem conexão para buscar o vídeo.".to_string())?;

    match resp.status().as_u16() {
        200 => {}
        401 | 403 => return Err("Vídeo privado ou com restrição.".into()),
        400 | 404 => return Err("Vídeo não encontrado.".into()),
        code => return Err(format!("O YouTube respondeu com erro {code}.")),
    }
    let data: OEmbed = resp
        .json()
        .await
        .map_err(|_| "Resposta inesperada do YouTube.".to_string())?;

    Ok(VideoInfo {
        thumbnail: thumbnail_url(&id),
        id,
        url,
        title: data.title,
        channel: data.author_name,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thumbnail_from_id() {
        assert_eq!(thumbnail_url("dQw4w9WgXcQ"), "https://i.ytimg.com/vi/dQw4w9WgXcQ/mqdefault.jpg");
    }

    #[tokio::test]
    async fn rejects_non_youtube_without_network() {
        assert_eq!(video_info("https://vimeo.com/1").await.unwrap_err(), "Esse link não parece ser do YouTube.");
    }
}
