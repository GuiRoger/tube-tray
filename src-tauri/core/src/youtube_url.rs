use url::Url;

/// Valida e normaliza uma URL de vídeo do YouTube.
///
/// Aceita `youtube.com/watch?v=`, `youtu.be/<id>`, `/shorts/<id>`, `/live/<id>`,
/// `/embed/<id>` e `music.youtube.com`. Remove parâmetros de playlist para que
/// só o vídeo seja baixado (playlists ficam fora do MVP).
/// Retorna `None` se não for um link de vídeo do YouTube.
pub fn normalize(input: &str) -> Option<String> {
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

    is_video_id(&id).then(|| format!("https://www.youtube.com/watch?v={id}"))
}

fn is_video_id(id: &str) -> bool {
    id.len() == 11
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

#[cfg(test)]
mod tests {
    use super::normalize;

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
}
