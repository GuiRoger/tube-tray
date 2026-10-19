//! Fila de downloads: um worker, um vídeo por vez, cancelável.
//!
//! A fila não conhece a UI. Quem a cria passa:
//! - `EnvProvider`: resolve yt-dlp, ffmpeg e pasta de destino na hora de
//!   começar cada job (assim mudanças de configuração valem para o próximo);
//! - `Sink`: recebe cada atualização de job (o app repassa como evento).
//!
//! Com [`DownloadManager::with_history`], os jobs finalizados são gravados em
//! JSON e voltam na próxima abertura do app.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use futures_util::future::BoxFuture;
use serde::{Deserialize, Serialize};
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::profile::Profile;
use crate::progress::{Snapshot, Stage};
use crate::runner::{self, RunSpec};
use crate::youtube_url;

/// Quantos jobs finalizados o histórico guarda.
pub const HISTORY_LIMIT: usize = 100;

#[derive(Debug, Clone)]
pub struct RunEnv {
    pub ytdlp: PathBuf,
    pub ffmpeg: Option<PathBuf>,
    pub out_dir: PathBuf,
}

pub type EnvProvider = Arc<dyn Fn() -> BoxFuture<'static, Result<RunEnv, String>> + Send + Sync>;
pub type Sink = Arc<dyn Fn(&Job) + Send + Sync>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: Uuid,
    pub url: String,
    pub profile_id: String,
    /// epoch em milissegundos
    pub created_at: u64,
    #[serde(flatten)]
    pub snapshot: Snapshot,
    #[serde(skip)]
    profile: Profile,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum EnqueueError {
    #[error("Isso não parece um link de vídeo do YouTube.")]
    InvalidUrl,
    #[error("Este vídeo já está na fila.")]
    Duplicate,
}

struct Inner {
    jobs: Mutex<Vec<Job>>,
    cancels: Mutex<HashMap<Uuid, CancellationToken>>,
    wake: Notify,
    env: EnvProvider,
    sink: Sink,
    history: Option<PathBuf>,
}

#[derive(Clone)]
pub struct DownloadManager {
    inner: Arc<Inner>,
}

fn queued_snapshot() -> Snapshot {
    Snapshot {
        stage: Stage::Queued,
        title: None,
        percent: 0.0,
        speed: None,
        eta: None,
        file: None,
        size: None,
        error: None,
        missing: false,
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn is_active(stage: Stage) -> bool {
    matches!(stage, Stage::Queued | Stage::Starting | Stage::Downloading | Stage::Merging)
}

/// Arquivo de um job concluído que não existe mais no disco.
fn file_missing(job: &Job) -> bool {
    job.snapshot.stage == Stage::Done
        && job.snapshot.file.as_deref().is_some_and(|f| !Path::new(f).exists())
}

impl DownloadManager {
    pub fn new(env: EnvProvider, sink: Sink) -> Self {
        Self::build(env, sink, None, Vec::new())
    }

    /// Igual a [`new`](Self::new), mas lê e grava o histórico em `path`.
    pub fn with_history(env: EnvProvider, sink: Sink, path: PathBuf) -> Self {
        let mut jobs: Vec<Job> = std::fs::read(&path)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        jobs.retain(|j| !is_active(j.snapshot.stage));
        for j in &mut jobs {
            j.profile = Profile::by_id(&j.profile_id).unwrap_or_default();
            j.snapshot.missing = file_missing(j);
        }
        Self::build(env, sink, Some(path), jobs)
    }

    fn build(env: EnvProvider, sink: Sink, history: Option<PathBuf>, jobs: Vec<Job>) -> Self {
        Self {
            inner: Arc::new(Inner {
                jobs: Mutex::new(jobs),
                cancels: Mutex::new(HashMap::new()),
                wake: Notify::new(),
                env,
                sink,
                history,
            }),
        }
    }

    /// Grava os jobs finalizados (os mais recentes) no histórico.
    fn save_history(&self) {
        let Some(path) = &self.inner.history else { return };
        let finished: Vec<Job> = {
            let jobs = self.inner.jobs.lock().unwrap();
            let done: Vec<&Job> = jobs.iter().filter(|j| !is_active(j.snapshot.stage)).collect();
            let skip = done.len().saturating_sub(HISTORY_LIMIT);
            done.into_iter().skip(skip).cloned().collect()
        };
        let Ok(bytes) = serde_json::to_vec(&finished) else { return };
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let tmp = path.with_extension("tmp");
        if std::fs::write(&tmp, bytes).is_ok() {
            let _ = std::fs::rename(&tmp, path);
        }
    }

    /// Loop do worker. O app chama uma vez, num task do runtime async.
    pub async fn run_worker(self) {
        loop {
            let next = {
                let jobs = self.inner.jobs.lock().unwrap();
                jobs.iter().find(|j| j.snapshot.stage == Stage::Queued).cloned()
            };
            match next {
                Some(job) => self.process(job).await,
                None => self.inner.wake.notified().await,
            }
        }
    }

    async fn process(&self, job: Job) {
        let token = CancellationToken::new();
        self.inner.cancels.lock().unwrap().insert(job.id, token.clone());

        let mut starting = queued_snapshot();
        starting.stage = Stage::Starting;
        self.update(job.id, starting.clone());

        let final_snap = tokio::select! {
            env = (self.inner.env)() => match env {
                Ok(env) => {
                    let spec = RunSpec {
                        ytdlp: env.ytdlp,
                        ffmpeg: env.ffmpeg,
                        out_dir: env.out_dir,
                        profile: job.profile.clone(),
                        url: job.url.clone(),
                    };
                    let this = self.clone();
                    let id = job.id;
                    runner::run(&spec, token.clone(), move |s| this.update(id, s.clone())).await
                }
                Err(msg) => Snapshot { stage: Stage::Failed, error: Some(msg), ..starting },
            },
            _ = token.cancelled() => Snapshot { stage: Stage::Canceled, ..starting },
        };

        self.update(job.id, final_snap);
        self.inner.cancels.lock().unwrap().remove(&job.id);
    }

    fn update(&self, id: Uuid, snap: Snapshot) {
        let (job, finished) = {
            let mut jobs = self.inner.jobs.lock().unwrap();
            let Some(job) = jobs.iter_mut().find(|j| j.id == id) else { return };
            // o fim do download chega uma vez só para a UI (sem notificação dupla)
            let repeated = !is_active(snap.stage) && job.snapshot.stage == snap.stage;
            // mantém o título já conhecido se a atualização não trouxer um
            let title = snap.title.clone().or_else(|| job.snapshot.title.clone());
            job.snapshot = Snapshot { title, ..snap };
            if repeated {
                return;
            }
            (job.clone(), !is_active(job.snapshot.stage))
        };
        (self.inner.sink)(&job);
        if finished {
            self.save_history();
        }
    }

    /// Coloca um vídeo na fila. `title` (opcional) é o título já conhecido
    /// pela prévia, para a lista mostrar o nome desde o início.
    pub fn enqueue(&self, url: &str, profile: Profile, title: Option<String>) -> Result<Job, EnqueueError> {
        let url = youtube_url::normalize(url).ok_or(EnqueueError::InvalidUrl)?;
        let job = {
            let mut jobs = self.inner.jobs.lock().unwrap();
            if jobs.iter().any(|j| j.url == url && is_active(j.snapshot.stage)) {
                return Err(EnqueueError::Duplicate);
            }
            let job = Job {
                id: Uuid::new_v4(),
                url,
                profile_id: profile.id.clone(),
                created_at: now_ms(),
                snapshot: Snapshot { title: title.filter(|t| !t.trim().is_empty()), ..queued_snapshot() },
                profile,
            };
            jobs.push(job.clone());
            job
        };
        (self.inner.sink)(&job);
        self.inner.wake.notify_one();
        Ok(job)
    }

    /// Cancela um job na fila ou em andamento. Retorna `false` se não existe
    /// ou já terminou.
    pub fn cancel(&self, id: Uuid) -> bool {
        if let Some(token) = self.inner.cancels.lock().unwrap().get(&id) {
            token.cancel();
            return true;
        }
        let job = {
            let mut jobs = self.inner.jobs.lock().unwrap();
            match jobs.iter_mut().find(|j| j.id == id && j.snapshot.stage == Stage::Queued) {
                Some(j) => {
                    j.snapshot.stage = Stage::Canceled;
                    j.clone()
                }
                None => return false,
            }
        };
        (self.inner.sink)(&job);
        self.save_history();
        true
    }

    /// Recoloca na fila um job que falhou, foi cancelado ou cujo arquivo
    /// foi excluído.
    pub fn retry(&self, id: Uuid) -> bool {
        let job = {
            let mut jobs = self.inner.jobs.lock().unwrap();
            match jobs.iter_mut().find(|j| {
                j.id == id
                    && (matches!(j.snapshot.stage, Stage::Failed | Stage::Canceled)
                        || (j.snapshot.stage == Stage::Done && j.snapshot.missing))
            }) {
                Some(j) => {
                    let title = j.snapshot.title.take();
                    j.snapshot = Snapshot { title, ..queued_snapshot() };
                    j.clone()
                }
                None => return false,
            }
        };
        (self.inner.sink)(&job);
        self.inner.wake.notify_one();
        true
    }

    /// Confere se os arquivos baixados ainda existem e marca os excluídos
    /// (ou os que voltaram). Devolve os jobs que mudaram.
    pub fn refresh_missing(&self) -> Vec<Job> {
        let changed: Vec<Job> = {
            let mut jobs = self.inner.jobs.lock().unwrap();
            jobs.iter_mut()
                .filter(|j| j.snapshot.stage == Stage::Done)
                .filter_map(|j| {
                    let missing = file_missing(j);
                    (missing != j.snapshot.missing).then(|| {
                        j.snapshot.missing = missing;
                        j.clone()
                    })
                })
                .collect()
        };
        for job in &changed {
            (self.inner.sink)(job);
        }
        if !changed.is_empty() {
            self.save_history();
        }
        changed
    }

    /// Remove um job já finalizado da lista. Retorna `false` se ele ainda
    /// estiver ativo ou não existir.
    pub fn remove(&self, id: Uuid) -> bool {
        let removed = {
            let mut jobs = self.inner.jobs.lock().unwrap();
            let before = jobs.len();
            jobs.retain(|j| j.id != id || is_active(j.snapshot.stage));
            jobs.len() != before
        };
        if removed {
            self.save_history();
        }
        removed
    }

    /// Remove da lista os jobs concluídos, com falha ou cancelados.
    pub fn clear_finished(&self) {
        self.inner
            .jobs
            .lock()
            .unwrap()
            .retain(|j| is_active(j.snapshot.stage));
        self.save_history();
    }

    /// Jobs do mais novo para o mais antigo.
    pub fn list(&self) -> Vec<Job> {
        let mut v = self.inner.jobs.lock().unwrap().clone();
        v.reverse();
        v
    }

    pub fn has_active(&self) -> bool {
        self.inner
            .jobs
            .lock()
            .unwrap()
            .iter()
            .any(|j| is_active(j.snapshot.stage))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    const URL: &str = "https://youtu.be/dQw4w9WgXcQ";

    fn manager_with(env: EnvProvider) -> (DownloadManager, Arc<Mutex<Vec<Job>>>) {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let s = seen.clone();
        let m = DownloadManager::new(env, Arc::new(move |j: &Job| s.lock().unwrap().push(j.clone())));
        (m, seen)
    }

    fn failing_env() -> EnvProvider {
        Arc::new(|| Box::pin(async { Err::<RunEnv, _>("sem yt-dlp".to_string()) }))
    }

    async fn wait_for(m: &DownloadManager, stage: Stage) {
        for _ in 0..100 {
            if m.list()[0].snapshot.stage == stage {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("job não chegou em {stage:?}");
    }

    #[test]
    fn enqueue_validates_and_dedupes() {
        let (m, _) = manager_with(failing_env());
        assert_eq!(m.enqueue("oi", Profile::default(), None).unwrap_err(), EnqueueError::InvalidUrl);
        let job = m.enqueue(URL, Profile::default(), Some("Título da prévia".into())).unwrap();
        assert_eq!(job.url, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
        assert_eq!(job.snapshot.title.as_deref(), Some("Título da prévia"));
        assert_eq!(
            m.enqueue("https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=1", Profile::default(), None)
                .unwrap_err(),
            EnqueueError::Duplicate
        );
    }

    #[test]
    fn cancel_queued_job() {
        let (m, _) = manager_with(failing_env());
        let job = m.enqueue(URL, Profile::default(), None).unwrap();
        assert!(m.cancel(job.id));
        assert_eq!(m.list()[0].snapshot.stage, Stage::Canceled);
        assert!(!m.has_active());
        assert!(m.retry(job.id));
        assert_eq!(m.list()[0].snapshot.stage, Stage::Queued);
    }

    #[tokio::test]
    async fn env_failure_marks_job_failed_once() {
        let (m, seen) = manager_with(failing_env());
        tokio::spawn(m.clone().run_worker());
        m.enqueue(URL, Profile::default(), None).unwrap();
        wait_for(&m, Stage::Failed).await;
        tokio::time::sleep(Duration::from_millis(30)).await;
        let job = &m.list()[0];
        assert_eq!(job.snapshot.error.as_deref(), Some("sem yt-dlp"));
        // o estado final chega à UI uma única vez
        let stages: Vec<_> = seen.lock().unwrap().iter().map(|j| j.snapshot.stage).collect();
        assert_eq!(stages, [Stage::Queued, Stage::Starting, Stage::Failed]);
        let id = m.list()[0].id;
        assert!(m.remove(id));
        assert!(m.list().is_empty());
    }

    /// Ambiente que "baixa" criando o arquivo final na hora.
    fn fake_download_env(dir: PathBuf) -> EnvProvider {
        Arc::new(move || {
            let dir = dir.clone();
            Box::pin(async move { Ok(RunEnv { ytdlp: dir.join("nao-existe"), ffmpeg: None, out_dir: dir }) })
        })
    }

    #[tokio::test]
    async fn history_persists_and_detects_deleted_files() {
        let dir = tempfile::tempdir().unwrap();
        let history = dir.path().join("history.json");
        let sink: Sink = Arc::new(|_| {});
        let m = DownloadManager::with_history(failing_env(), sink.clone(), history.clone());
        tokio::spawn(m.clone().run_worker());
        let job = m.enqueue(URL, Profile::default(), Some("Vídeo".into())).unwrap();
        wait_for(&m, Stage::Failed).await;
        assert!(history.is_file());

        // simula um download concluído cujo arquivo existe
        let file = dir.path().join("Vídeo [dQw4w9WgXcQ].mp4");
        std::fs::write(&file, b"mp4").unwrap();
        {
            let mut jobs = m.inner.jobs.lock().unwrap();
            jobs[0].snapshot.stage = Stage::Done;
            jobs[0].snapshot.file = Some(file.to_string_lossy().into_owned());
        }
        m.save_history();

        // reabre o app: o histórico volta
        let m2 = DownloadManager::with_history(fake_download_env(dir.path().into()), sink, history.clone());
        let loaded = m2.list();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, job.id);
        assert_eq!(loaded[0].snapshot.title.as_deref(), Some("Vídeo"));
        assert!(!loaded[0].snapshot.missing);
        assert!(m2.refresh_missing().is_empty());

        // usuário apaga o arquivo
        std::fs::remove_file(&file).unwrap();
        let changed = m2.refresh_missing();
        assert_eq!(changed.len(), 1);
        assert!(changed[0].snapshot.missing);
        assert!(m2.refresh_missing().is_empty(), "só avisa uma vez");
        // um vídeo excluído pode ser baixado de novo
        assert!(m2.retry(job.id));
        assert_eq!(m2.list()[0].snapshot.stage, Stage::Queued);
        assert!(!m2.list()[0].snapshot.missing);
    }
}
