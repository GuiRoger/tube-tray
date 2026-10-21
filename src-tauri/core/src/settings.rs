use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Configurações do usuário, salvas como JSON na pasta de dados do app.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub out_dir: PathBuf,
    pub launch_at_login: bool,
    pub profile_id: String,
    /// preencher a URL a partir da área de transferência ao abrir a janela
    pub read_clipboard: bool,
    pub notify: bool,
    /// aviso de uso responsável já confirmado (DESIGN_SYSTEM §12)
    pub accepted_notice: bool,
    /// downloads ao mesmo tempo (1–5)
    pub max_parallel: usize,
    /// atalho global "baixar o link copiado"
    pub shortcut_enabled: bool,
    /// no formato do Tauri, ex.: "CommandOrControl+Alt+Y"
    pub shortcut: String,
}

/// Ctrl+Alt+Y no Windows, ⌥⌘Y no Mac.
pub const DEFAULT_SHORTCUT: &str = "CommandOrControl+Alt+Y";

impl Default for Settings {
    fn default() -> Self {
        Self {
            out_dir: PathBuf::new(),
            launch_at_login: false,
            profile_id: "mp4-h264".into(),
            read_clipboard: true,
            notify: true,
            accepted_notice: false,
            max_parallel: crate::queue::DEFAULT_PARALLEL,
            shortcut_enabled: true,
            shortcut: DEFAULT_SHORTCUT.into(),
        }
    }
}

impl Settings {
    pub const FILE_NAME: &'static str = "settings.json";

    /// Pasta padrão: `<Downloads>/TubeTray`.
    pub fn with_default_out_dir(mut self, downloads: &Path) -> Self {
        if self.out_dir.as_os_str().is_empty() {
            self.out_dir = downloads.join("TubeTray");
        }
        self
    }

    /// Lê o arquivo; se não existir ou estiver corrompido, usa o padrão.
    pub fn load(data_dir: &Path) -> Self {
        std::fs::read(data_dir.join(Self::FILE_NAME))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, data_dir: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(data_dir)?;
        let tmp = data_dir.join(format!("{}.tmp", Self::FILE_NAME));
        std::fs::write(&tmp, serde_json::to_vec_pretty(self)?)?;
        std::fs::rename(tmp, data_dir.join(Self::FILE_NAME))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_defaults() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(Settings::load(dir.path()), Settings::default());

        let s = Settings::default().with_default_out_dir(Path::new("/home/gui/Downloads"));
        assert_eq!(s.out_dir, Path::new("/home/gui/Downloads/TubeTray"));
        s.save(dir.path()).unwrap();
        assert_eq!(Settings::load(dir.path()), s);

        // campos ausentes num arquivo antigo assumem o padrão
        std::fs::write(dir.path().join(Settings::FILE_NAME), br#"{"outDir":"/x"}"#).unwrap();
        let old = Settings::load(dir.path());
        assert_eq!(old.out_dir, Path::new("/x"));
        assert!(old.notify);
        assert_eq!(old.max_parallel, 2);
        assert_eq!(old.shortcut, DEFAULT_SHORTCUT);
    }
}
