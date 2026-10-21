use url::Url;

/// Valida e normaliza uma URL de vídeo do YouTube.
///
/// Aceita `youtube.com/watch?v=`, `youtu.be/<id>`, `/shorts/<id>`, `/live/<id>`,
/// `/embed/<id>` e `music.youtube.com`. Remove parâmetros de playlist: o
/// download é sempre de um vídeo; playlists viram vários jobs (ver
/// [`playlist_url`]).
/// Retorna `None` se não for um link de vídeo do YouTube.
pub fn normalize(input: &str) -> Option<String> {
    video_id(input).map(|id| format!("https://www.youtube.com/watch?v={id}"))
}

/// Extrai o id de 11 caracteres de um link de vídeo do YouTube.
pub fn video_id(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    let with_scheme = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    };
    let url = Url::parse(&with_scheme).ok()?;
    let host = url.host_str()?.to_ascii_lowercase();
    let host = host.strip_prefix("www.").unwrap_or(&host);
    let host = host.strip_prefix("m.").unwrap_or(host);

    let id = match host {
        "youtu.be" => url.path_segments()?.next().map(str::to_string),
        "youtube.com" | "music.youtube.com" | "youtube-nocookie.com" => {
            let mut segs = url.path_segments()?;
            match segs.next() {
                Some("watch") => url
                    .query_pairs()
                    .find(|(k, _)| k == "v")
                    .map(|(_, v)| v.into_owned()),
                Some("shorts") | Some("live") | Some("embed") | Some("v") => {
                    segs.next().map(str::to_string)
                }
                _ => None,
            }
        }
        _ => None,
    }?;

    is_video_id(&id).then_some(id)
}

fn parse(input: &str) -> Option<Url> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    let with_scheme = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    };
    let url = Url::parse(&with_scheme).ok()?;
    let host = url.host_str()?.to_ascii_lowercase();
    let host = host.strip_prefix("www.").unwrap_or(&host);
    let host = host.strip_prefix("m.").unwrap_or(host);
    matches!(host, "youtube.com" | "music.youtube.com" | "youtu.be").then_some(url)
}

/// Id da playlist (`list=`) de um link do YouTube, se houver.
pub fn playlist_id(input: &str) -> Option<String> {
    let url = parse(input)?;
    let id = url.query_pairs().find(|(k, _)| k == "list").map(|(_, v)| v.into_owned())?;
    let ok = (2..=64).contains(&id.len())
        && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    ok.then_some(id)
}

/// Link canônico para listar uma playlist com o yt-dlp. Mixes (`RD…`)
/// só existem junto de um vídeo, então mantêm o `v=`.
pub fn playlist_url(input: &str) -> Option<String> {
    let list = playlist_id(input)?;
    match video_id(input) {
        Some(v) if list.starts_with("RD") => Some(format!("https://www.youtube.com/watch?v={v}&list={list}")),
        _ if list.starts_with("RD") => None,
        _ => Some(format!("https://www.youtube.com/playlist?list={list}")),
    }
}

fn is_video_id(id: &str) -> bool {
    id.len() == 11
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

#[cfg(test)]
mod tests {
    use super::{normalize, playlist_id, playlist_url};

    const ID: &str = "dQw4w9WgXcQ";

    fn expected() -> Option<String> {
        Some(format!("https://www.youtube.com/watch?v={ID}"))
    }

    #[test]
    fn accepts_common_forms() {
        for input in [
            format!("https://www.youtube.com/watch?v={ID}"),
            format!("https://youtube.com/watch?v={ID}&t=42s"),
            format!("https://www.youtube.com/watch?v={ID}&list=PL123&index=2"),
            format!("youtube.com/watch?v={ID}"),
            format!("https://youtu.be/{ID}"),
            format!("https://youtu.be/{ID}?si=abc"),
            format!("https://www.youtube.com/shorts/{ID}"),
            format!("https://m.youtube.com/watch?v={ID}"),
            format!("https://music.youtube.com/watch?v={ID}"),
            format!("https://www.youtube.com/live/{ID}"),
            format!("  https://youtu.be/{ID}  "),
        ] {
            assert_eq!(normalize(&input), expected(), "input: {input}");
        }
    }

    #[test]
    fn rejects_non_videos() {
        for input in [
            "",
            "hello",
            "https://vimeo.com/123",
            "https://www.youtube.com/",
            "https://www.youtube.com/@canal",
            "https://www.youtube.com/playlist?list=PL123",
            "https://www.youtube.com/watch?v=curto",
            "https://notyoutube.com/watch?v=dQw4w9WgXcQ",
        ] {
            assert_eq!(normalize(input), None, "input: {input}");
        }
    }

    #[test]
    fn playlists() {
        assert_eq!(playlist_id("https://www.youtube.com/playlist?list=PLabc_123-x"), Some("PLabc_123-x".into()));
        assert_eq!(
            playlist_url(&format!("https://www.youtube.com/watch?v={ID}&list=PL123&index=2")),
            Some("https://www.youtube.com/playlist?list=PL123".into())
        );
        assert_eq!(
            playlist_url(&format!("https://youtu.be/{ID}?list=RDabc")),
            Some(format!("https://www.youtube.com/watch?v={ID}&list=RDabc"))
        );
        assert_eq!(playlist_url("https://www.youtube.com/playlist?list=RDabc"), None);
        assert_eq!(playlist_id(&format!("https://youtu.be/{ID}")), None);
        assert_eq!(playlist_id("https://vimeo.com/x?list=PL123"), None);
        assert_eq!(playlist_id("https://www.youtube.com/playlist?list=PL<script>"), None);
    }
}
