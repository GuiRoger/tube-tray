mod commands;
mod notify;
mod state;
mod tray;
mod window;

use std::collections::HashSet;
use std::sync::{Arc, LazyLock, Mutex};

use tauri::{Emitter, Manager, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;
use tubetray_core::queue::{is_active, EnvProvider, Sink};
use tubetray_core::{tools, DownloadManager, Job, RunEnv, Settings};
use uuid::Uuid;

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
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let data_dir = app.path().app_data_dir()?;
            notify::init(&data_dir);
            let downloads = app
                .path()
                .download_dir()
                .or_else(|_| app.path().home_dir())?;
            let settings = Arc::new(Mutex::new(
                Settings::load(&data_dir).with_default_out_dir(&downloads),
            ));

            let manager = DownloadManager::with_history(
                env_provider(app.handle().clone(), data_dir.clone(), settings.clone()),
                job_sink(app.handle().clone()),
                data_dir.join("history.json"),
            );
            // o histórico carregado já foi notificado em sessões anteriores
            FINISHED.lock().unwrap().extend(manager.list().iter().map(|j| j.id));
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
            commands::remove_job,
            commands::open_file,
            commands::video_info,
            commands::refresh_files,
            commands::list_jobs,
            commands::get_settings,
            commands::set_settings,
            commands::reveal_file,
            commands::open_out_dir,
            commands::ytdlp_version,
            commands::update_ytdlp,
            commands::hide_window,
            commands::set_pinned,
            commands::quit_app,
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

/// Jobs cujo fim já foi avisado (evita notificar de novo quando só muda,
/// por exemplo, o aviso de arquivo excluído).
static FINISHED: LazyLock<Mutex<HashSet<Uuid>>> = LazyLock::new(Default::default);

/// Repassa cada atualização de job para a UI, atualiza o ícone da bandeja
/// e notifica no fim do download.
fn job_sink(app: tauri::AppHandle) -> Sink {
    Arc::new(move |job: &Job| {
        let _ = app.emit(EV_JOB, job);
        let Some(state) = app.try_state::<AppState>() else { return };

        let stage = job.snapshot.stage;
        let just_finished = {
            let mut finished = FINISHED.lock().unwrap();
            if is_active(stage) {
                finished.remove(&job.id); // retry: avisa de novo no fim
                false
            } else {
                finished.insert(job.id)
            }
        };

        if just_finished {
            tray::note_finished(stage, window::is_visible(&app));
            // avisa sempre que um download termina (com a janela aberta ou não)
            if state.settings.lock().unwrap().notify {
                notify::job_finished(&app, job);
            }
        }
        tray::refresh(&app);
    })
}
