//! Fila de downloads: um worker, um vídeo por vez, cancelável.
//!
//! A fila não conhece a UI. Quem a cria passa:
//! - `EnvProvider`: resolve yt-dlp, ffmpeg e pasta de destino na hora de
//!   começar cada job (assim mudanças de configuração valem para o próximo);
//! - `Sink`: recebe cada atualização de job (o app repassa como evento).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use futures_util::future::BoxFuture;
use serde::Serialize;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::profile::Profile;
use crate::progress::{Snapshot, Stage};
use crate::runner::{self, RunSpec};
use crate::youtube_url;

#[derive(Debug, Clone)]
pub struct RunEnv {
    pub ytdlp: PathBuf,
    pub ffmpeg: Option<PathBuf>,
    pub out_dir: PathBuf,
}

pub type EnvProvider = Arc<dyn Fn() -> BoxFuture<'static, Result<RunEnv, String>> + Send + Sync>;
pub type Sink = Arc<dyn Fn(&Job) + Send + Sync>;

#[derive(Debug, Clone, Serialize)]
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
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn is_active(stage: Stage) -> bool {
    matches!(stage, Stage::Queued | Stage::Starting | Stage::Downloading | Stage::Merging)
}

impl DownloadManager {
    pub fn new(env: EnvProvider, sink: Sink) -> Self {
        Self {
            inner: Arc::new(Inner {
                jobs: Mutex::new(Vec::new()),
                cancels: Mutex::new(HashMap::new()),
                wake: Notify::new(),
                env,
                sink,
            }),
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
        let job = {
            let mut jobs = self.inner.jobs.lock().unwrap();
            let Some(job) = jobs.iter_mut().find(|j| j.id == id) else { return };
            // mantém o título já conhecido se a atualização não trouxer um
            let title = snap.title.clone().or_else(|| job.snapshot.title.clone());
            job.snapshot = Snapshot { title, ..snap };
            job.clone()
        };
        (self.inner.sink)(&job);
    }

    pub fn enqueue(&self, url: &str, profile: Profile) -> Result<Job, EnqueueError> {
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
                snapshot: queued_snapshot(),
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
        true
    }

    /// Recoloca na fila um job que falhou ou foi cancelado.
    pub fn retry(&self, id: Uuid) -> bool {
        let job = {
            let mut jobs = self.inner.jobs.lock().unwrap();
            match jobs.iter_mut().find(|j| {
                j.id == id && matches!(j.snapshot.stage, Stage::Failed | Stage::Canceled)
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

    /// Remove um job já finalizado da lista. Retorna `false` se ele ainda
    /// estiver ativo ou não existir.
    pub fn remove(&self, id: Uuid) -> bool {
        let mut jobs = self.inner.jobs.lock().unwrap();
        let before = jobs.len();
        jobs.retain(|j| j.id != id || is_active(j.snapshot.stage));
        jobs.len() != before
    }

    /// Remove da lista os jobs concluídos, com falha ou cancelados.
    pub fn clear_finished(&self) {
        self.inner
            .jobs
            .lock()
            .unwrap()
            .retain(|j| is_active(j.snapshot.stage));
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

    #[test]
    fn enqueue_validates_and_dedupes() {
        let (m, _) = manager_with(failing_env());
        assert_eq!(m.enqueue("oi", Profile::default()).unwrap_err(), EnqueueError::InvalidUrl);
        let job = m.enqueue(URL, Profile::default()).unwrap();
        assert_eq!(job.url, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
        assert_eq!(
            m.enqueue("https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=1", Profile::default())
                .unwrap_err(),
            EnqueueError::Duplicate
        );
    }

    #[test]
    fn cancel_queued_job() {
        let (m, _) = manager_with(failing_env());
        let job = m.enqueue(URL, Profile::default()).unwrap();
        assert!(m.cancel(job.id));
        assert_eq!(m.list()[0].snapshot.stage, Stage::Canceled);
        assert!(!m.has_active());
        assert!(m.retry(job.id));
        assert_eq!(m.list()[0].snapshot.stage, Stage::Queued);
    }

    #[tokio::test]
    async fn env_failure_marks_job_failed() {
        let (m, seen) = manager_with(failing_env());
        tokio::spawn(m.clone().run_worker());
        m.enqueue(URL, Profile::default()).unwrap();
        for _ in 0..50 {
            if m.list()[0].snapshot.stage == Stage::Failed {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        let job = &m.list()[0];
        assert_eq!(job.snapshot.stage, Stage::Failed);
        assert_eq!(job.snapshot.error.as_deref(), Some("sem yt-dlp"));
        let stages: Vec<_> = seen.lock().unwrap().iter().map(|j| j.snapshot.stage).collect();
        assert_eq!(stages, [Stage::Queued, Stage::Starting, Stage::Failed]);
        let id = m.list()[0].id;
        assert!(m.remove(id));
        assert!(m.list().is_empty());
    }
}
