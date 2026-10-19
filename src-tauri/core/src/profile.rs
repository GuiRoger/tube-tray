use serde::{Deserialize, Serialize};

/// Um perfil é o conjunto de argumentos de formato passado ao yt-dlp.
/// Novos formatos (só áudio, melhor qualidade...) entram como novos perfis,
/// sem mexer no runner nem na UI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub args: Vec<String>,
}

impl Profile {
    /// Equivalente ao comando usado hoje:
    /// `yt-dlp -S "vcodec:h264,res,acodec:m4a" --merge-output-format mp4 <url>`
    pub fn mp4_h264() -> Self {
        Self {
            id: "mp4-h264".into(),
            name: "MP4 (H.264 + M4A)".into(),
            args: vec![
                "-S".into(),
                "vcodec:h264,res,acodec:m4a".into(),
                "--merge-output-format".into(),
                "mp4".into(),
            ],
        }
    }

    pub fn builtin() -> Vec<Self> {
        vec![Self::mp4_h264()]
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
    }
}
