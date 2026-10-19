use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_opener::OpenerExt;

use crate::state::AppState;
use crate::window;

const TRAY_ID: &str = "tubetray";

const ICON_IDLE: &[u8] = include_bytes!("../icons/tray.png");
const ICON_ACTIVE: &[u8] = include_bytes!("../icons/tray-active.png");

pub fn create<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let paste = MenuItem::with_id(app, "paste", "Baixar link copiado", true, None::<&str>)?;
    let open = MenuItem::with_id(app, "open", "Abrir TubeTray", true, None::<&str>)?;
    let folder = MenuItem::with_id(app, "folder", "Abrir pasta de downloads", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&paste, &open, &folder, &sep, &quit])?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(Image::from_bytes(ICON_IDLE)?)
        .icon_as_template(cfg!(target_os = "macos"))
        .tooltip("TubeTray")
        .menu(&menu)
        // clique esquerdo abre a janela; o menu fica no clique direito
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "paste" => enqueue_clipboard(app),
            "open" => window::show(app, true),
            "folder" => open_out_dir(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                window::toggle(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

/// Troca o ícone da bandeja entre ocioso e baixando.
pub fn set_active<R: Runtime>(app: &AppHandle<R>, active: bool) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else { return };
    let bytes = if active { ICON_ACTIVE } else { ICON_IDLE };
    if let Ok(img) = Image::from_bytes(bytes) {
        let _ = tray.set_icon(Some(img));
        let _ = tray.set_icon_as_template(cfg!(target_os = "macos"));
    }
    let _ = tray.set_tooltip(Some(if active { "TubeTray — baixando" } else { "TubeTray" }));
}

fn enqueue_clipboard<R: Runtime>(app: &AppHandle<R>) {
    let text = app.clipboard().read_text().unwrap_or_default();
    let state = app.state::<AppState>();
    let profile = crate::commands::current_profile(&state);
    if state.manager.enqueue(&text, profile).is_err() {
        // link inválido ou repetido: abre a janela para o usuário ver/colar
        window::show(app, true);
    }
}

fn open_out_dir<R: Runtime>(app: &AppHandle<R>) {
    let dir = app.state::<AppState>().settings.lock().unwrap().out_dir.clone();
    let _ = std::fs::create_dir_all(&dir);
    let _ = app.opener().open_path(dir.to_string_lossy(), None::<&str>);
}
