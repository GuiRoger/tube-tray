//! Localiza, baixa e atualiza os binários externos.
//!
//! - **yt-dlp** fica na pasta de dados do app (gravável), é baixado do GitHub
//!   no primeiro uso e se atualiza com `yt-dlp -U`.
//! - **ffmpeg** vai embutido no app como sidecar, ao lado do executável.

use std::path::{Path, PathBuf};
use std::process::Stdio;

use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

use crate::runner::hidden_command;

const RELEASE_BASE: &str = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/";

static INSTALL_LOCK: Mutex<()> = Mutex::const_new(());

/// Nome do binário do yt-dlp na release do GitHub para esta plataforma.
pub fn ytdlp_asset_name() -> &'static str {
    if cfg!(windows) {
        "yt-dlp.exe"
    } else if cfg!(target_os = "macos") {
        "yt-dlp_macos"
    } else if cfg!(target_arch = "aarch64") {
        "yt-dlp_linux_aarch64"
    } else {
        "yt-dlp_linux"
    }
}

pub fn exe_name(base: &str) -> String {
    if cfg!(windows) {
        format!("{base}.exe")
    } else {
        base.to_string()
    }
}

pub fn ytdlp_path(bin_dir: &Path) -> PathBuf {
    bin_dir.join(exe_name("yt-dlp"))
}

/// Caminho do ffmpeg embutido (o Tauri copia sidecars para junto do executável,
/// sem o sufixo do target triple). `None` se não estiver lá: o yt-dlp então
/// procura o ffmpeg no PATH.
pub fn bundled_ffmpeg() -> Option<PathBuf> {
    let dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let p = dir.join(exe_name("ffmpeg"));
    p.is_file().then_some(p)
}

/// Garante que o yt-dlp exista em `bin_dir`, baixando se necessário.
/// `on_progress` recebe (baixados, total) em bytes.
pub async fn ensure_ytdlp(
    bin_dir: &Path,
    mut on_progress: impl FnMut(u64, Option<u64>),
) -> Result<PathBuf, String> {
    let target = ytdlp_path(bin_dir);
    let _guard = INSTALL_LOCK.lock().await;
    if target.is_file() {
        return Ok(target);
    }
    tokio::fs::create_dir_all(bin_dir)
        .await
        .map_err(|e| format!("Não foi possível criar {}: {e}", bin_dir.display()))?;

    let url = format!("{RELEASE_BASE}{}", ytdlp_asset_name());
    let resp = reqwest::Client::builder()
        .user_agent(concat!("TubeTray/", env!("CARGO_PKG_VERSION")))
        .build()
        .map(|c| c.get(&url))
        .map_err(|e| e.to_string())?
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| format!("Não foi possível baixar o yt-dlp: {e}"))?;

    let total = resp.content_length();
    let tmp = target.with_extension("download");
    let mut file = tokio::fs::File::create(&tmp)
        .await
        .map_err(|e| format!("Não foi possível gravar o yt-dlp: {e}"))?;
    let mut done = 0u64;
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Download do yt-dlp interrompido: {e}"))?;
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;
        done += chunk.len() as u64;
        on_progress(done, total);
    }
    file.flush().await.map_err(|e| e.to_string())?;
    drop(file);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        tokio::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755))
            .await
            .map_err(|e| e.to_string())?;
    }
    tokio::fs::rename(&tmp, &target).await.map_err(|e| e.to_string())?;
    Ok(target)
}

async fn run_capture(program: &Path, args: &[&str]) -> Result<String, String> {
    let out = hidden_command(program)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .await
        .map_err(|e| format!("Não foi possível executar o yt-dlp: {e}"))?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    if out.status.success() {
        Ok(text)
    } else {
        Err(text.trim().lines().last().unwrap_or("erro desconhecido").to_string())
    }
}

pub async fn ytdlp_version(ytdlp: &Path) -> Result<String, String> {
    Ok(run_capture(ytdlp, &["--version"]).await?.trim().to_string())
}

/// Atualiza o yt-dlp para a última versão estável. Devolve a versão final.
pub async fn update_ytdlp(ytdlp: &Path) -> Result<String, String> {
    let _guard = INSTALL_LOCK.lock().await;
    run_capture(ytdlp, &["-U"]).await?;
    drop(_guard);
    ytdlp_version(ytdlp).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_per_platform() {
        let p = ytdlp_path(Path::new("bin"));
        if cfg!(windows) {
            assert!(p.ends_with("yt-dlp.exe"));
        } else {
            assert!(p.ends_with("yt-dlp"));
        }
        assert!(!ytdlp_asset_name().is_empty());
    }
}
