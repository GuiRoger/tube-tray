use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager, Runtime, WebviewWindow, Window};
use tauri_plugin_positioner::{Position, WindowExt};

pub const MAIN: &str = "main";

/// Quando "fixada", a janela não some ao perder o foco
/// (usado enquanto um diálogo de pasta está aberto, por exemplo).
static PINNED: AtomicBool = AtomicBool::new(false);

/// Momento em que a janela sumiu por perda de foco. Clicar no ícone da
/// bandeja tira o foco da janela antes do clique chegar; sem isso o clique
/// esconderia e reabriria a janela na mesma hora.
static LAST_AUTO_HIDE: Mutex<Option<Instant>> = Mutex::new(None);

pub fn set_pinned(pinned: bool) {
    PINNED.store(pinned, Ordering::Relaxed);
}

fn main_window<R: Runtime>(app: &AppHandle<R>) -> Option<WebviewWindow<R>> {
    app.get_webview_window(MAIN)
}

pub fn is_visible<R: Runtime>(app: &AppHandle<R>) -> bool {
    main_window(app).and_then(|w| w.is_visible().ok()).unwrap_or(false)
}

/// Mostra a janela. `near_tray` posiciona junto ao ícone da bandeja;
/// sem isso (ex.: segunda instância) ela abre no canto inferior/superior direito.
pub fn show<R: Runtime>(app: &AppHandle<R>, near_tray: bool) {
    let Some(w) = main_window(app) else { return };
    let pos = if near_tray {
        Position::TrayCenter
    } else if cfg!(target_os = "macos") {
        Position::TopRight
    } else {
        Position::BottomRight
    };
    // TrayCenter falha se ainda não houve evento da bandeja; cai no canto
    if w.move_window(pos).is_err() {
        let _ = w.move_window(Position::BottomRight);
    }
    let _ = w.show();
    let _ = w.unminimize();
    let _ = w.set_focus();
    let _ = w.emit("window://shown", ());
}

pub fn toggle<R: Runtime>(app: &AppHandle<R>) {
    let just_hidden = LAST_AUTO_HIDE
        .lock()
        .unwrap()
        .is_some_and(|t| t.elapsed() < Duration::from_millis(300));
    if just_hidden {
        return;
    }
    if is_visible(app) {
        if let Some(w) = main_window(app) {
            let _ = w.hide();
        }
    } else {
        show(app, true);
    }
}

pub fn hide_if_unpinned<R: Runtime>(win: &Window<R>) {
    if win.label() == MAIN && !PINNED.load(Ordering::Relaxed) {
        *LAST_AUTO_HIDE.lock().unwrap() = Some(Instant::now());
        let _ = win.hide();
    }
}
