//! "Colar e baixar": pega o link copiado e põe na fila sem abrir a janela.
//! Usado pelo atalho global e pelo item do menu da bandeja.

use std::time::Duration;

use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use tubetray_core::{youtube_url, EnqueueError, Settings};

use crate::state::AppState;
use crate::{notify, window};

pub fn paste_and_download<R: Runtime>(app: &AppHandle<R>) {
    let text = app.clipboard().read_text().unwrap_or_default().trim().to_string();

    // só playlist (sem vídeo): a escolha dos itens precisa da janela
    if youtube_url::video_id(&text).is_none() && youtube_url::playlist_id(&text).is_some() {
        window::show(app, true);
        return;
    }
    if youtube_url::video_id(&text).is_none() {
        notify::info(
            app,
            "Nenhum link do YouTube copiado",
            "Copie o link de um vídeo e use o atalho de novo.",
        );
        return;
    }

    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        // o título deixa a notificação e a lista mais claras; sem ele, segue
        let title = tokio::time::timeout(Duration::from_secs(4), tubetray_core::metadata::video_info(&text))
            .await
            .ok()
            .and_then(Result::ok)
            .map(|v| v.title);
        let state = app.state::<AppState>();
        let profile = crate::commands::current_profile(&state);
        let fmt = profile.name.clone();
        match state.manager.enqueue(&text, profile, title.clone()) {
            Ok(_) => notify::info(
                &app,
                "Baixando",
                &format!("{}\n{fmt}", title.as_deref().unwrap_or("Vídeo do link copiado")),
            ),
            Err(EnqueueError::Duplicate) => notify::info(&app, "Já está na fila", title.as_deref().unwrap_or(&text)),
            Err(e) => notify::info(&app, "Não foi possível baixar", &e.to_string()),
        }
    });
}

/// (Re)registra o atalho global conforme os ajustes. Devolve erro em
/// português se o atalho for inválido ou já estiver em uso por outro app.
pub fn apply_shortcut<R: Runtime>(app: &AppHandle<R>, settings: &Settings) -> Result<(), String> {
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();
    if !settings.shortcut_enabled || settings.shortcut.trim().is_empty() {
        return Ok(());
    }
    gs.on_shortcut(settings.shortcut.as_str(), |app, _shortcut, event| {
        if event.state == ShortcutState::Pressed {
            paste_and_download(app);
        }
    })
    .map_err(|e| {
        log::warn!("atalho {}: {e}", settings.shortcut);
        "Esse atalho não está disponível (inválido ou já usado por outro app).".to_string()
    })
}
