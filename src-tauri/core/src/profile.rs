use serde::{Deserialize, Serialize};

/// Vídeo ou só áudio (o seletor Vídeo | Áudio da UI).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MediaKind {
    Video,
    Audio,
}

/// Um perfil é o conjunto de argumentos de formato passado ao yt-dlp.
/// Novos formatos entram como novos perfis, sem mexer no runner.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub kind: MediaKind,
    /// nome curto para o seletor ("Compatível · MP4")
    pub name: String,
    /// explicação de uma linha
    pub hint: String,
    pub args: Vec<String>,
}

fn args(a: &[&str]) -> Vec<String> {
    a.iter().map(|s| s.to_string()).collect()
}

impl Profile {
    pub const DEFAULT_ID: &'static str = "mp4-h264";

    /// Equivalente ao comando de sempre:
    /// `yt-dlp -S "vcodec:h264,res,acodec:m4a" --merge-output-format mp4 <url>`
    pub fn mp4_h264() -> Self {
        Self {
            id: "mp4-h264".into(),
            kind: MediaKind::Video,
            name: "Compatível · MP4".into(),
            hint: "H.264 + AAC, abre em qualquer lugar (até 1080p)".into(),
            args: args(&["-S", "vcodec:h264,res,acodec:m4a", "--merge-output-format", "mp4"]),
        }
    }

    /// Maior resolução disponível (4K/8K, HDR), em qualquer codec.
    /// Fica em MP4 quando os codecs permitem; senão MKV.
    pub fn best() -> Self {
        Self {
            id: "best".into(),
            kind: MediaKind::Video,
            name: "Melhor qualidade".into(),
            hint: "Até 4K/8K · MP4 ou MKV (AV1/VP9)".into(),
            args: args(&[
                "-f",
                "bv*+ba/b",
                "-S",
                "res,fps,hdr:12,acodec:m4a",
                "--merge-output-format",
                "mp4/mkv",
            ]),
        }
    }

    /// Só o áudio em M4A (AAC): sem reconversão quando o YouTube já tem M4A.
    pub fn audio_m4a() -> Self {
        Self {
            id: "audio-m4a".into(),
            kind: MediaKind::Audio,
            name: "M4A · alta".into(),
            hint: "AAC, sem perda extra".into(),
            args: args(&[
                "-f",
                "ba[ext=m4a]/ba/b",
                "-x",
                "--audio-format",
                "m4a",
                "--embed-metadata",
            ]),
        }
    }

    /// Só o áudio em MP3 (VBR de melhor qualidade).
    pub fn audio_mp3() -> Self {
        Self {
            id: "audio-mp3".into(),
            kind: MediaKind::Audio,
            name: "MP3".into(),
            hint: "VBR ~245 kbps, toca em qualquer aparelho".into(),
            args: args(&[
                "-f",
                "ba/b",
                "-x",
                "--audio-format",
                "mp3",
                "--audio-quality",
                "0",
                "--embed-metadata",
            ]),
        }
    }

    /// Extensões possíveis do arquivo final, da mais provável à menos.
    pub fn extensions(&self) -> &'static [&'static str] {
        match self.id.as_str() {
            "best" => &["mp4", "mkv", "webm"],
            "audio-m4a" => &["m4a", "mp3", "opus"],
            "audio-mp3" => &["mp3"],
            _ => &["mp4", "mkv"],
        }
    }

    pub fn builtin() -> Vec<Self> {
        vec![Self::mp4_h264(), Self::best(), Self::audio_m4a(), Self::audio_mp3()]
    }

    pub fn by_id(id: &str) -> Option<Self> {
        Self::builtin().into_iter().find(|p| p.id == id)
    }
}

impl Default for Profile {
    fn default() -> Self {
        Self::mp4_h264()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_profile_matches_current_command() {
        let p = Profile::default();
        assert_eq!(
            p.args,
            ["-S", "vcodec:h264,res,acodec:m4a", "--merge-output-format", "mp4"]
        );
        assert_eq!(Profile::by_id("mp4-h264"), Some(p));
        assert_eq!(Profile::DEFAULT_ID, Profile::default().id);
    }

    #[test]
    fn builtin_profiles() {
        let ids: Vec<_> = Profile::builtin().into_iter().map(|p| p.id).collect();
        assert_eq!(ids, ["mp4-h264", "best", "audio-m4a", "audio-mp3"]);
        for p in Profile::builtin() {
            let audio = p.args.iter().any(|a| a == "-x");
            assert_eq!(audio, p.kind == MediaKind::Audio, "{}", p.id);
        }
        assert!(Profile::by_id("nao-existe").is_none());
    }
}
