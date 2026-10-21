//! Comandos chamados pela UI (`invoke`). Erros voltam como texto pronto
//! para exibir.

use tauri::{AppHandle, Manager, State};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_opener::OpenerExt;
use serde::Deserialize;
use tauri::Emitter;
use tubetray_core::{tools, Job, PlaylistInfo, Profile, Settings, VideoInfo};
use uuid::Uuid;

use crate::state::AppState;
use crate::window;

type CmdResult<T> = Result<T, String>;

pub fn current_profile(state: &AppState) -> Profile {
    let id = state.settings.lock().unwrap().profile_id.clone();
    Profile::by_id(&id).unwrap_or_default()
}

/// Perfil pedido pela UI; sem id (ou id desconhecido), o padrão dos ajustes.
fn pick_profile(state: &AppState, id: Option<String>) -> Profile {
    id.and_then(|id| Profile::by_id(&id)).unwrap_or_else(|| current_profile(state))
}

/// Formatos disponíveis (Vídeo: compatível, melhor; Áudio: M4A, MP3).
#[tauri::command]
pub fn profiles() -> Vec<Profile> {
    Profile::builtin()
}

#[tauri::command]
pub fn enqueue(
    state: State<'_, AppState>,
    url: String,
    title: Option<String>,
    profile_id: Option<String>,
) -> CmdResult<Job> {
    let profile = pick_profile(&state, profile_id);
    state.manager.enqueue(&url, profile, title).map_err(|e| e.to_string())
}

#[derive(Deserialize)]
pub struct BatchItem {
    url: String,
    title: Option<String>,
}

/// Coloca vários vídeos (itens escolhidos de uma playlist) na fila, numa
/// subpasta com o nome da playlist. Repetidos são ignorados. Devolve
/// quantos entraram.
#[tauri::command]
pub fn enqueue_many(
    state: State<'_, AppState>,
    items: Vec<BatchItem>,
    profile_id: Option<String>,
    subdir: Option<String>,
) -> CmdResult<usize> {
    let profile = pick_profile(&state, profile_id);
    let added = items
        .into_iter()
        .filter(|it| {
            state
                .manager
                .enqueue_in(&it.url, profile.clone(), it.title.clone(), subdir.clone())
                .is_ok()
        })
        .count();
    if added == 0 {
        return Err("Esses vídeos já estão na fila.".into());
    }
    Ok(added)
}

/// Vídeos de uma playlist (baixa o yt-dlp antes, se for a primeira vez).
#[tauri::command]
pub async fn playlist_info(app: AppHandle, url: String) -> CmdResult<PlaylistInfo> {
    let bin_dir = app.state::<AppState>().data_dir.join("bin");
    let mut last_pct = u64::MAX;
    let ytdlp = tools::ensure_ytdlp(&bin_dir, |done, total| {
        let pct = total.map(|t| done * 100 / t.max(1)).unwrap_or(0);
        if pct != last_pct {
            last_pct = pct;
            let _ = app.emit(crate::EV_YTDLP_DOWNLOAD, (done, total));
        }
    })
    .await?;
    tubetray_core::playlist::playlist_info(&ytdlp, &url).await
}

/// Desliga o atalho global enquanto o usuário grava um novo (senão o
/// sistema "engole" as teclas do atalho atual) e religa depois.
#[tauri::command]
pub fn pause_shortcut(app: AppHandle, state: State<'_, AppState>, paused: bool) {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;
    if paused {
        let _ = app.global_shortcut().unregister_all();
    } else {
        let settings = state.settings.lock().unwrap().clone();
        let _ = crate::quick::apply_shortcut(&app, &settings);
    }
}

/// `None` se o atalho global está registrado (ou desligado); senão o erro.
#[tauri::command]
pub fn shortcut_status(app: AppHandle, state: State<'_, AppState>) -> Option<String> {
    let settings = state.settings.lock().unwrap().clone();
    crate::quick::apply_shortcut(&app, &settings).err()
}

/// Título, canal e miniatura do vídeo colado (prévia antes de baixar).
#[tauri::command]
pub async fn video_info(url: String) -> CmdResult<VideoInfo> {
    tubetray_core::metadata::video_info(&url).await
}

/// Confere se os vídeos baixados ainda existem; os excluídos chegam à UI
/// pelo evento de job.
#[tauri::command]
pub fn refresh_files(state: State<'_, AppState>) {
    state.manager.refresh_missing();
}

#[tauri::command]
pub fn cancel(state: State<'_, AppState>, id: Uuid) -> bool {
    state.manager.cancel(id)
}

#[tauri::command]
pub fn cancel_all(state: State<'_, AppState>) -> usize {
    state.manager.cancel_all()
}

#[tauri::command]
pub fn retry(state: State<'_, AppState>, id: Uuid) -> bool {
    state.manager.retry(id)
}

#[tauri::command]
pub fn remove_job(state: State<'_, AppState>, id: Uuid) -> bool {
    state.manager.remove(id)
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

    let mut settings = settings;
    settings.max_parallel = settings.max_parallel.clamp(1, tubetray_core::queue::MAX_PARALLEL);
    if Profile::by_id(&settings.profile_id).is_none() {
        settings.profile_id = Profile::DEFAULT_ID.into();
    }

    // atalho: se o novo não puder ser registrado, volta ao anterior
    let previous = state.settings.lock().unwrap().clone();
    let shortcut_changed =
        previous.shortcut != settings.shortcut || previous.shortcut_enabled != settings.shortcut_enabled;
    if shortcut_changed {
        if let Err(e) = crate::quick::apply_shortcut(&app, &settings) {
            let _ = crate::quick::apply_shortcut(&app, &previous);
            return Err(e);
        }
    }

    settings
        .save(&state.data_dir)
        .map_err(|e| format!("Não foi possível salvar as configurações: {e}"))?;
    state.manager.set_max_parallel(settings.max_parallel);
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
pub fn open_file(app: AppHandle, path: String) -> CmdResult<()> {
    app.opener()
        .open_path(&path, None::<&str>)
        .map_err(|e| format!("Não foi possível abrir o arquivo: {e}"))
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

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}
