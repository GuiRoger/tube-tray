//! Comandos chamados pela UI (`invoke`). Erros voltam como texto pronto
//! para exibir.

use tauri::{AppHandle, Manager, State};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_opener::OpenerExt;
use tubetray_core::{tools, Job, Profile, Settings};
use uuid::Uuid;

use crate::state::AppState;
use crate::window;

type CmdResult<T> = Result<T, String>;

pub fn current_profile(state: &AppState) -> Profile {
    let id = state.settings.lock().unwrap().profile_id.clone();
    Profile::by_id(&id).unwrap_or_default()
}

#[tauri::command]
pub fn enqueue(state: State<'_, AppState>, url: String) -> CmdResult<Job> {
    let profile = current_profile(&state);
    state.manager.enqueue(&url, profile).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cancel(state: State<'_, AppState>, id: Uuid) -> bool {
    state.manager.cancel(id)
}

#[tauri::command]
pub fn retry(state: State<'_, AppState>, id: Uuid) -> bool {
    state.manager.retry(id)
}

#[tauri::command]
pub fn clear_finished(state: State<'_, AppState>) -> Vec<Job> {
    state.manager.clear_finished();
    state.manager.list()
}

#[tauri::command]
pub fn list_jobs(state: State<'_, AppState>) -> Vec<Job> {
    state.manager.list()
}

#[tauri::command]
pub fn get_settings(app: AppHandle, state: State<'_, AppState>) -> Settings {
    let mut s = state.settings.lock().unwrap().clone();
    // o estado real do autostart vem do sistema, não do arquivo
    if let Ok(enabled) = app.autolaunch().is_enabled() {
        s.launch_at_login = enabled;
    }
    s
}

#[tauri::command]
pub fn set_settings(app: AppHandle, state: State<'_, AppState>, settings: Settings) -> CmdResult<Settings> {
    if settings.out_dir.as_os_str().is_empty() {
        return Err("Escolha uma pasta de destino.".into());
    }
    let autolaunch = app.autolaunch();
    let result = if settings.launch_at_login {
        autolaunch.enable()
    } else {
        autolaunch.disable()
    };
    result.map_err(|e| format!("Não foi possível alterar o início automático: {e}"))?;

    settings
        .save(&state.data_dir)
        .map_err(|e| format!("Não foi possível salvar as configurações: {e}"))?;
    *state.settings.lock().unwrap() = settings.clone();
    Ok(settings)
}

#[tauri::command]
pub fn reveal_file(app: AppHandle, path: String) -> CmdResult<()> {
    app.opener()
        .reveal_item_in_dir(&path)
        .map_err(|e| format!("Não foi possível abrir a pasta: {e}"))
}

#[tauri::command]
pub fn open_out_dir(app: AppHandle, state: State<'_, AppState>) -> CmdResult<()> {
    let dir = state.settings.lock().unwrap().out_dir.clone();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    app.opener()
        .open_path(dir.to_string_lossy(), None::<&str>)
        .map_err(|e| format!("Não foi possível abrir a pasta: {e}"))
}

/// Versão do yt-dlp instalado, ou `None` se ainda não foi baixado.
#[tauri::command]
pub async fn ytdlp_version(state: State<'_, AppState>) -> CmdResult<Option<String>> {
    let path = state.ytdlp_path();
    if !path.is_file() {
        return Ok(None);
    }
    tools::ytdlp_version(&path).await.map(Some)
}

/// Baixa o yt-dlp se faltar; senão atualiza. Devolve a versão final.
#[tauri::command]
pub async fn update_ytdlp(app: AppHandle) -> CmdResult<String> {
    let state = app.state::<AppState>();
    if state.manager.has_active() {
        return Err("Aguarde os downloads em andamento terminarem.".into());
    }
    let bin_dir = state.data_dir.join("bin");
    let path = tools::ensure_ytdlp(&bin_dir, |_, _| {}).await?;
    tools::update_ytdlp(&path).await
}

#[tauri::command]
pub fn hide_window(app: AppHandle) {
    if let Some(w) = app.get_webview_window(window::MAIN) {
        let _ = w.hide();
    }
}

/// Mantém a janela aberta mesmo sem foco (ex.: diálogo de escolher pasta).
#[tauri::command]
pub fn set_pinned(pinned: bool) {
    window::set_pinned(pinned);
}
