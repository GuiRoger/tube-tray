//! Ícone e menu da bandeja (Windows) / barra de menu (macOS).
//!
//! Estados do ícone (DESIGN_SYSTEM §6.2, versão com fundo vermelho): ocioso, baixando (anel de progresso
//! em 11 quadros de 10 em 10%), concluído não visto (ponto) e erro não visto
//! ("!"). Os avisos de concluído/erro somem quando a janela é aberta.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_opener::OpenerExt;
use tubetray_core::Stage;

use crate::state::AppState;
use crate::window;

const TRAY_ID: &str = "tubetray";

/// Quadros na ordem: idle, done, error, progress 0..=100 (de 10 em 10).
macro_rules! tray_set {
    ($dir:literal) => {
        [
            include_bytes!(concat!("../icons/tray/", $dir, "/idle.png")).as_slice(),
            include_bytes!(concat!("../icons/tray/", $dir, "/done.png")).as_slice(),
            include_bytes!(concat!("../icons/tray/", $dir, "/error.png")).as_slice(),
            include_bytes!(concat!("../icons/tray/", $dir, "/progress-000.png")).as_slice(),
            include_bytes!(concat!("../icons/tray/", $dir, "/progress-010.png")).as_slice(),
            include_bytes!(concat!("../icons/tray/", $dir, "/progress-020.png")).as_slice(),
            include_bytes!(concat!("../icons/tray/", $dir, "/progress-030.png")).as_slice(),
            include_bytes!(concat!("../icons/tray/", $dir, "/progress-040.png")).as_slice(),
            include_bytes!(concat!("../icons/tray/", $dir, "/progress-050.png")).as_slice(),
            include_bytes!(concat!("../icons/tray/", $dir, "/progress-060.png")).as_slice(),
            include_bytes!(concat!("../icons/tray/", $dir, "/progress-070.png")).as_slice(),
            include_bytes!(concat!("../icons/tray/", $dir, "/progress-080.png")).as_slice(),
            include_bytes!(concat!("../icons/tray/", $dir, "/progress-090.png")).as_slice(),
            include_bytes!(concat!("../icons/tray/", $dir, "/progress-100.png")).as_slice(),
        ]
    };
}

// Ícones com fundo vermelho (squircle do ícone do app + glifo branco).
// macOS usa a versão 64 px (o sistema reduz para 18 pt na barra de menu).
#[cfg(target_os = "macos")]
static FRAMES: [&[u8]; 14] = tray_set!("macos");
#[cfg(not(target_os = "macos"))]
static FRAMES: [&[u8]; 14] = tray_set!("windows");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Icon {
    Idle,
    Done,
    Error,
    /// 0..=10 (quadro do anel)
    Progress(u8),
}

impl Icon {
    fn index(self) -> usize {
        match self {
            Icon::Idle => 0,
            Icon::Done => 1,
            Icon::Error => 2,
            Icon::Progress(f) => 3 + f.min(10) as usize,
        }
    }
}

static UNSEEN_DONE: AtomicBool = AtomicBool::new(false);
static UNSEEN_ERROR: AtomicBool = AtomicBool::new(false);
static CURRENT: Mutex<Option<Icon>> = Mutex::new(None);

fn frames() -> &'static [&'static [u8]; 14] {
    &FRAMES
}

pub fn create<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Abrir TubeTray", true, None::<&str>)?;
    let paste = MenuItem::with_id(app, "paste", "Colar e baixar", true, None::<&str>)?;
    let folder = MenuItem::with_id(app, "folder", "Abrir pasta de downloads", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Ajustes…", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&open, &paste, &sep1, &folder, &settings, &sep2, &quit])?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(Image::from_bytes(frames()[Icon::Idle.index()])?)
        // colorido: não é template, o macOS não deve inverter
        .icon_as_template(false)
        .tooltip("TubeTray")
        .menu(&menu)
        // clique esquerdo abre o popover; o menu fica no clique direito
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" => window::show(app, true),
            "paste" => enqueue_clipboard(app),
            "folder" => open_out_dir(app),
            "settings" => {
                window::show(app, true);
                let _ = app.emit("ui://open-settings", ());
            }
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
    *CURRENT.lock().unwrap() = Some(Icon::Idle);
    Ok(())
}

/// Registra o fim de um job para o aviso "não visto" do ícone.
pub fn note_finished(stage: Stage, window_visible: bool) {
    if window_visible {
        return;
    }
    match stage {
        Stage::Done => UNSEEN_DONE.store(true, Ordering::Relaxed),
        Stage::Failed => UNSEEN_ERROR.store(true, Ordering::Relaxed),
        _ => {}
    }
}

/// Chamado quando a janela abre: limpa os avisos e redesenha o ícone.
pub fn mark_seen<R: Runtime>(app: &AppHandle<R>) {
    UNSEEN_DONE.store(false, Ordering::Relaxed);
    UNSEEN_ERROR.store(false, Ordering::Relaxed);
    refresh(app);
}

/// Recalcula o estado do ícone a partir da fila. Só troca a imagem quando o
/// quadro muda (o anel anda de 10 em 10%), o que mantém a atualização bem
/// abaixo do limite de 250 ms do design.
pub fn refresh<R: Runtime>(app: &AppHandle<R>) {
    let Some(state) = app.try_state::<AppState>() else { return };
    let jobs = state.manager.list();
    let running = jobs
        .iter()
        .find(|j| matches!(j.snapshot.stage, Stage::Starting | Stage::Downloading | Stage::Merging));
    let queued = jobs.iter().any(|j| j.snapshot.stage == Stage::Queued);

    let icon = if let Some(j) = running {
        Icon::Progress((j.snapshot.percent / 10.0).floor().clamp(0.0, 10.0) as u8)
    } else if queued {
        Icon::Progress(0)
    } else if UNSEEN_ERROR.load(Ordering::Relaxed) {
        Icon::Error
    } else if UNSEEN_DONE.load(Ordering::Relaxed) {
        Icon::Done
    } else {
        Icon::Idle
    };

    {
        let mut cur = CURRENT.lock().unwrap();
        if *cur == Some(icon) {
            return;
        }
        *cur = Some(icon);
    }

    let Some(tray) = app.tray_by_id(TRAY_ID) else { return };
    if let Ok(img) = Image::from_bytes(frames()[icon.index()]) {
        let _ = tray.set_icon(Some(img));
    }
    let tip = match (icon, running) {
        (Icon::Progress(_), Some(j)) => format!("TubeTray — baixando {}%", j.snapshot.percent.floor()),
        (Icon::Progress(_), None) => "TubeTray — na fila".into(),
        (Icon::Error, _) => "TubeTray — um download falhou".into(),
        (Icon::Done, _) => "TubeTray — download concluído".into(),
        (Icon::Idle, _) => "TubeTray".into(),
    };
    let _ = tray.set_tooltip(Some(tip));
}

fn enqueue_clipboard<R: Runtime>(app: &AppHandle<R>) {
    let text = app.clipboard().read_text().unwrap_or_default();
    let state = app.state::<AppState>();
    let profile = crate::commands::current_profile(&state);
    if state.manager.enqueue(&text, profile, None).is_err() {
        // link inválido ou repetido: abre o popover para o usuário ver/colar
        window::show(app, true);
    }
}

fn open_out_dir<R: Runtime>(app: &AppHandle<R>) {
    let dir = app.state::<AppState>().settings.lock().unwrap().out_dir.clone();
    let _ = std::fs::create_dir_all(&dir);
    let _ = app.opener().open_path(dir.to_string_lossy(), None::<&str>);
}
