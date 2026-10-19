mod commands;
mod state;
mod tray;
mod window;

use std::sync::{Arc, Mutex};

use tauri::{Emitter, Manager, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_notification::NotificationExt;
use tubetray_core::queue::{EnvProvider, Sink};
use tubetray_core::{tools, DownloadManager, Job, RunEnv, Settings, Stage};

use crate::state::AppState;

/// Eventos enviados para a UI.
pub const EV_JOB: &str = "jobs://update";
pub const EV_YTDLP_DOWNLOAD: &str = "tools://ytdlp-download";

pub fn run() {
    tauri::Builder::default()
        // um segundo clique no atalho só mostra a janela da instância aberta
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            window::show(app, false);
        }))
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let data_dir = app.path().app_data_dir()?;
            let downloads = app
                .path()
                .download_dir()
                .or_else(|_| app.path().home_dir())?;
            let settings = Arc::new(Mutex::new(
                Settings::load(&data_dir).with_default_out_dir(&downloads),
            ));

            let manager = DownloadManager::new(
                env_provider(app.handle().clone(), data_dir.clone(), settings.clone()),
                job_sink(app.handle().clone()),
            );
            tauri::async_runtime::spawn(manager.clone().run_worker());

            app.manage(AppState { manager, settings, data_dir });
            tray::create(app.handle())?;
            Ok(())
        })
        .on_window_event(|win, event| match event {
            // fechar só esconde: o app continua na bandeja
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = win.hide();
            }
            // comportamento de popover: some ao clicar fora
            WindowEvent::Focused(false) => window::hide_if_unpinned(win),
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::enqueue,
            commands::cancel,
            commands::retry,
            commands::clear_finished,
            commands::list_jobs,
            commands::get_settings,
            commands::set_settings,
            commands::reveal_file,
            commands::open_out_dir,
            commands::ytdlp_version,
            commands::update_ytdlp,
            commands::hide_window,
            commands::set_pinned,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o TubeTray");
}

/// Resolve yt-dlp, ffmpeg e pasta de destino no início de cada download.
fn env_provider(
    app: tauri::AppHandle,
    data_dir: std::path::PathBuf,
    settings: Arc<Mutex<Settings>>,
) -> EnvProvider {
    Arc::new(move || {
        let app = app.clone();
        let bin_dir = data_dir.join("bin");
        let out_dir = settings.lock().unwrap().out_dir.clone();
        Box::pin(async move {
            let mut last_pct = u64::MAX;
            let ytdlp = tools::ensure_ytdlp(&bin_dir, |done, total| {
                let pct = total.map(|t| done * 100 / t.max(1)).unwrap_or(0);
                if pct != last_pct {
                    last_pct = pct;
                    let _ = app.emit(EV_YTDLP_DOWNLOAD, (done, total));
                }
            })
            .await?;
            Ok(RunEnv { ytdlp, ffmpeg: tools::bundled_ffmpeg(), out_dir })
        })
    })
}

/// Repassa cada atualização de job para a UI, atualiza o ícone da bandeja
/// e notifica no fim do download.
fn job_sink(app: tauri::AppHandle) -> Sink {
    Arc::new(move |job: &Job| {
        let _ = app.emit(EV_JOB, job);

        if let Some(state) = app.try_state::<AppState>() {
            tray::set_active(&app, state.manager.has_active());

            let notify = state.settings.lock().unwrap().notify;
            let window_visible = window::is_visible(&app);
            if notify && !window_visible {
                let title = job.snapshot.title.clone().unwrap_or_else(|| job.url.clone());
                let body = match job.snapshot.stage {
                    Stage::Done => Some(("Download concluído", title)),
                    Stage::Failed => Some((
                        "Falha no download",
                        job.snapshot.error.clone().unwrap_or(title),
                    )),
                    _ => None,
                };
                if let Some((t, b)) = body {
                    let _ = app.notification().builder().title(t).body(b).show();
                }
            }
        }
    })
}
