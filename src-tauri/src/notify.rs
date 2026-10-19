//! Notificação do sistema ao fim de cada download, com ícone de
//! sucesso (verde) ou falha (vermelho).
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
    let file = job.snapshot.file.clone();
    let identifier = app.config().identifier.clone();

    // mostrar a notificação pode bloquear (COM / NSUserNotification)
    std::thread::spawn(move || {
        if let Err(e) = show(&identifier, head, &video, &detail, icon.as_deref(), file) {
            log::warn!("notificação falhou: {e}");
        }
    });
}

#[cfg(windows)]
fn show(
    identifier: &str,
    head: &str,
    video: &str,
    detail: &str,
    icon: Option<&Path>,
    file: Option<String>,
) -> Result<(), String> {
    use tauri_winrt_notification::{IconCrop, Sound, Toast};

    // Instalado, o atalho do app tem o AppUserModelID = identifier.
    // Em `tauri dev` não há atalho, então o toast sai como PowerShell.
    let app_id = if tauri::is_dev() { Toast::POWERSHELL_APP_ID } else { identifier };
    let mut toast = Toast::new(app_id)
        .title(head)
        .text1(video)
        .text2(detail)
        .sound(Some(Sound::Default));
    if let Some(icon) = icon {
        toast = toast.icon(icon, IconCrop::Square, "");
    }
    if let Some(file) = file {
        // clique na notificação de sucesso: mostra o arquivo no Explorer
        toast = toast.on_activated(move |_| {
            use std::os::windows::process::CommandExt;
            let _ = std::process::Command::new("explorer")
                .raw_arg(format!("/select,\"{file}\""))
                .spawn();
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
    _file: Option<String>,
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
    n.summary(head).body(&format!("{video}\n{detail}"));
    if let Some(icon) = icon {
        n.image_path(&icon.to_string_lossy());
    }
    n.show().map(|_| ()).map_err(|e| e.to_string())
}
