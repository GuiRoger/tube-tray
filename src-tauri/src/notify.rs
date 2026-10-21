//! Notificações do sistema: fim de cada download (ícone de sucesso verde
//! ou falha vermelho), resumo de playlist e avisos do atalho global.
//!
//! O plugin de notificação do Tauri ignora o ícone no Windows, então
//! a notificação é montada direto: toast WinRT no Windows e
//! notify-rust (NSUserNotification) no macOS.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use tauri::{AppHandle, Runtime};
use tubetray_core::{Job, Stage};

const ICON_SUCCESS: &[u8] = include_bytes!("../icons/notify/notify-success.png");
const ICON_FAILURE: &[u8] = include_bytes!("../icons/notify/notify-failure.png");

/// Os toasts precisam do ícone como arquivo; gravamos uma vez na pasta de dados.
static ICONS: OnceLock<(PathBuf, PathBuf)> = OnceLock::new();

pub fn init(data_dir: &Path) {
    let dir = data_dir.join("notify");
    let _ = std::fs::create_dir_all(&dir);
    let ok = dir.join("success.png");
    let err = dir.join("failure.png");
    for (path, bytes) in [(&ok, ICON_SUCCESS), (&err, ICON_FAILURE)] {
        if std::fs::read(path).map(|b| b != bytes).unwrap_or(true) {
            let _ = std::fs::write(path, bytes);
        }
    }
    let _ = ICONS.set((ok, err));
}

/// O que o clique na notificação faz (só no Windows; no Mac o clique abre o app).
#[derive(Debug, Clone)]
#[cfg_attr(not(windows), allow(dead_code))]
enum Click {
    /// mostra o arquivo selecionado na pasta
    Reveal(String),
    /// abre a pasta
    Open(String),
}

fn spawn_show<R: Runtime>(
    app: &AppHandle<R>,
    head: String,
    line1: String,
    line2: String,
    icon: Option<PathBuf>,
    click: Option<Click>,
) {
    let identifier = app.config().identifier.clone();
    // mostrar a notificação pode bloquear (COM / NSUserNotification)
    std::thread::spawn(move || {
        if let Err(e) = show(&identifier, &head, &line1, &line2, icon.as_deref(), click) {
            log::warn!("notificação falhou: {e}");
        }
    });
}

/// Aviso simples, sem ícone de status (atalho global, link inválido…).
pub fn info<R: Runtime>(app: &AppHandle<R>, head: &str, body: &str) {
    let (l1, l2) = body.split_once('\n').unwrap_or((body, ""));
    spawn_show(app, head.into(), l1.into(), l2.into(), None, None);
}

/// Resumo de uma playlist que terminou (um aviso só, em vez de um por vídeo).
pub fn batch_finished<R: Runtime>(app: &AppHandle<R>, name: &str, done: usize, failed: usize, folder: Option<String>) {
    let head = if failed == 0 { "Playlist baixada" } else { "Playlist baixada com falhas" };
    let videos = |n: usize| if n == 1 { "1 vídeo".to_string() } else { format!("{n} vídeos") };
    let detail = match (done, failed) {
        (d, 0) => format!("{} baixados.", videos(d)),
        (0, f) => format!("{} falharam.", videos(f)),
        (d, f) => format!("{} baixados · {} com falha.", videos(d), f),
    };
    let icon = ICONS.get().map(|(ok, err)| if done > 0 { ok.clone() } else { err.clone() });
    spawn_show(app, head.into(), name.into(), detail, icon, folder.map(Click::Open));
}

pub fn job_finished<R: Runtime>(app: &AppHandle<R>, job: &Job) {
    let success = match job.snapshot.stage {
        Stage::Done => true,
        Stage::Failed => false,
        _ => return,
    };
    let video = job.snapshot.title.clone().unwrap_or_else(|| job.url.clone());
    let head = if success { "Download concluído" } else { "Falha no download" };
    let detail = if success {
        "Terminou de baixar.".to_string()
    } else {
        job.snapshot.error.clone().unwrap_or_else(|| "Não foi possível baixar.".into())
    };
    let icon = ICONS.get().map(|(ok, err)| if success { ok.clone() } else { err.clone() });
    let click = job.snapshot.file.clone().map(Click::Reveal);
    spawn_show(app, head.into(), video, detail, icon, click);
}

#[cfg(windows)]
fn show(
    identifier: &str,
    head: &str,
    video: &str,
    detail: &str,
    icon: Option<&Path>,
    click: Option<Click>,
) -> Result<(), String> {
    use tauri_winrt_notification::{IconCrop, Sound, Toast};

    // Instalado, o atalho do app tem o AppUserModelID = identifier.
    // Em `tauri dev` não há atalho, então o toast sai como PowerShell.
    let app_id = if tauri::is_dev() { Toast::POWERSHELL_APP_ID } else { identifier };
    let mut toast = Toast::new(app_id).title(head).text1(video).sound(Some(Sound::Default));
    if !detail.is_empty() {
        toast = toast.text2(detail);
    }
    if let Some(icon) = icon {
        toast = toast.icon(icon, IconCrop::Square, "");
    }
    if let Some(click) = click {
        // clique: mostra o arquivo (ou abre a pasta) no Explorer
        toast = toast.on_activated(move |_| {
            use std::os::windows::process::CommandExt;
            let arg = match &click {
                Click::Reveal(file) => format!("/select,\"{file}\""),
                Click::Open(dir) => format!("\"{dir}\""),
            };
            let _ = std::process::Command::new("explorer").raw_arg(arg).spawn();
            Ok(())
        });
    }
    toast.show().map_err(|e| e.to_string())
}

#[cfg(not(windows))]
fn show(
    identifier: &str,
    head: &str,
    video: &str,
    detail: &str,
    icon: Option<&Path>,
    _click: Option<Click>,
) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let _ = notify_rust::set_application(if tauri::is_dev() {
            "com.apple.Terminal"
        } else {
            identifier
        });
    }
    #[cfg(not(target_os = "macos"))]
    let _ = identifier;

    let mut n = notify_rust::Notification::new();
    let body = if detail.is_empty() { video.to_string() } else { format!("{video}\n{detail}") };
    n.summary(head).body(&body);
    if let Some(icon) = icon {
        n.image_path(&icon.to_string_lossy());
    }
    n.show().map(|_| ()).map_err(|e| e.to_string())
}
