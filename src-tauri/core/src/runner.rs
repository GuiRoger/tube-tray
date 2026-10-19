//! Executa o yt-dlp para um único vídeo e transforma a saída em `Snapshot`s.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};

use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::profile::Profile;
use crate::progress::{
    friendly_error, parse_line, Snapshot, Stage, Tracker, FILE_TEMPLATE, META_TEMPLATE,
    PROGRESS_TEMPLATE,
};

/// Nome do arquivo final: título (limitado a 150 bytes) + id, para evitar
/// colisões e permitir limpar arquivos temporários do mesmo vídeo.
pub const OUTPUT_TEMPLATE: &str = "%(title).150B [%(id)s].%(ext)s";

/// Intervalo mínimo entre atualizações de progresso enviadas à UI.
const EMIT_EVERY: Duration = Duration::from_millis(150);

#[derive(Debug, Clone)]
pub struct RunSpec {
    pub ytdlp: PathBuf,
    pub ffmpeg: Option<PathBuf>,
    pub out_dir: PathBuf,
    pub profile: Profile,
    pub url: String,
}

impl RunSpec {
    pub fn args(&self) -> Vec<String> {
        let mut a: Vec<String> = self.profile.args.clone();
        a.extend(
            [
                "--newline",
                "--progress",
                "--no-colors",
                "--no-playlist",
                "--no-mtime",
                // o yt-dlp.exe escreveria no código de página do Windows
                "--encoding",
                "utf-8",
                "--progress-template",
                PROGRESS_TEMPLATE,
                "--print",
                META_TEMPLATE,
                "--print",
                FILE_TEMPLATE,
            ]
            .map(String::from),
        );
        if let Some(ff) = &self.ffmpeg {
            a.push("--ffmpeg-location".into());
            a.push(ff.to_string_lossy().into_owned());
        }
        a.push("-P".into());
        a.push(self.out_dir.to_string_lossy().into_owned());
        a.push("-o".into());
        a.push(OUTPUT_TEMPLATE.into());
        a.push("--".into());
        a.push(self.url.clone());
        a
    }
}

/// Cria um `Command` que não abre janela de console no Windows.
pub fn hidden_command(program: &Path) -> Command {
    let mut cmd = Command::new(program);
    cmd.env("PYTHONUTF8", "1").env("PYTHONIOENCODING", "utf-8");
    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(unix)]
    {
        // grupo de processos próprio para conseguir encerrar yt-dlp + ffmpeg juntos
        cmd.process_group(0);
    }
    cmd
}

/// Executa o download. `on_update` recebe cada mudança visível (com limite
/// de frequência para progresso). Sempre devolve o `Snapshot` final, cujo
/// `stage` é `Done`, `Failed` ou `Canceled`.
pub async fn run(
    spec: &RunSpec,
    cancel: CancellationToken,
    mut on_update: impl FnMut(&Snapshot),
) -> Snapshot {
    let mut tracker = Tracker::default();
    on_update(tracker.snapshot());

    if let Err(e) = tokio::fs::create_dir_all(&spec.out_dir).await {
        tracker.finish(Stage::Failed, Some(format!("Não foi possível criar a pasta de destino: {e}")));
        return tracker.snapshot().clone();
    }

    let mut child = match hidden_command(&spec.ytdlp)
        .args(spec.args())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            tracker.finish(Stage::Failed, Some(format!("Não foi possível iniciar o yt-dlp: {e}")));
            return tracker.snapshot().clone();
        }
    };

    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    if let Some(out) = child.stdout.take() {
        tokio::spawn(pump(out, tx.clone()));
    }
    if let Some(err) = child.stderr.take() {
        tokio::spawn(pump(err, tx.clone()));
    }
    drop(tx);

    let mut last_emit = Instant::now() - EMIT_EVERY;
    let mut last_stage = tracker.snapshot().stage;
    let mut canceled = false;

    loop {
        tokio::select! {
            _ = cancel.cancelled(), if !canceled => {
                canceled = true;
                kill_tree(&mut child).await;
            }
            line = rx.recv() => {
                let Some(line) = line else { break };
                if let Some(ev) = parse_line(&line) {
                    if tracker.apply(ev) {
                        let snap = tracker.snapshot();
                        if snap.stage != last_stage || last_emit.elapsed() >= EMIT_EVERY {
                            last_stage = snap.stage;
                            last_emit = Instant::now();
                            on_update(snap);
                        }
                    }
                } else {
                    log::debug!("yt-dlp: {line}");
                }
            }
        }
    }

    let status = child.wait().await;

    if canceled {
        if let Some(id) = &tracker.video_id {
            cleanup_partials(&spec.out_dir, id).await;
        }
        tracker.finish(Stage::Canceled, None);
    } else {
        match status {
            Ok(s) if s.success() => {
                tracker.finish(Stage::Done, None);
                confirm_output(&mut tracker, &spec.out_dir);
                if let Some(file) = tracker.snapshot().file.clone() {
                    let size = tokio::fs::metadata(&file).await.ok().map(|m| m.len());
                    tracker.set_size(size);
                }
            }
            Ok(s) => {
                let msg = tracker
                    .last_error
                    .as_deref()
                    .map(friendly_error)
                    .unwrap_or_else(|| format!("O yt-dlp terminou com erro ({s})."));
                tracker.finish(Stage::Failed, Some(msg));
            }
            Err(e) => tracker.finish(Stage::Failed, Some(format!("Falha ao aguardar o yt-dlp: {e}"))),
        }
    }
    let snap = tracker.snapshot().clone();
    on_update(&snap);
    snap
}

/// Garante que o caminho guardado é o do arquivo que existe no disco.
/// Se o caminho informado pelo yt-dlp não existir (ex.: acentos
/// corrompidos), procura pelo id do vídeo na pasta de destino.
fn confirm_output(tracker: &mut Tracker, out_dir: &Path) {
    let reported_ok = tracker
        .snapshot()
        .file
        .as_deref()
        .is_some_and(|f| Path::new(f).is_file());
    if reported_ok {
        return;
    }
    let Some(id) = tracker.video_id.clone() else { return };
    if let Some(found) = crate::files::find_by_id(out_dir, &id) {
        if tracker.snapshot().title.as_deref().is_none_or(crate::files::is_garbled) {
            tracker.set_title(crate::files::title_from_file(&found, &id));
        }
        tracker.set_file(Some(found.to_string_lossy().into_owned()));
    }
}

async fn pump(stream: impl AsyncRead + Unpin, tx: mpsc::UnboundedSender<String>) {
    let mut reader = BufReader::new(stream);
    let mut buf = Vec::with_capacity(256);
    loop {
        buf.clear();
        match reader.read_until(b'\n', &mut buf).await {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                // UTF-8; se não for, Windows-1252 (sem caracteres perdidos)
                let line = crate::files::decode_output(&buf);
                if tx.send(line).is_err() {
                    break;
                }
            }
        }
    }
}

/// Encerra o yt-dlp e os filhos dele (o executável do yt-dlp no Windows
/// sobe um segundo processo, e o ffmpeg roda como filho durante o merge).
async fn kill_tree(child: &mut tokio::process::Child) {
    #[cfg(windows)]
    if let Some(pid) = child.id() {
        let _ = hidden_command(Path::new("taskkill"))
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await;
    }
    #[cfg(unix)]
    if let Some(pid) = child.id() {
        // pid negativo = grupo inteiro
        unsafe {
            libc::kill(-(pid as i32), libc::SIGTERM);
        }
        if tokio::time::timeout(Duration::from_secs(3), child.wait()).await.is_ok() {
            return;
        }
    }
    let _ = child.kill().await;
}

/// Remove arquivos temporários de um download cancelado
/// (`.part`, `.ytdl`, `.fNNN.ext`, `.temp.ext`), preservando qualquer
/// arquivo final já existente do mesmo vídeo.
pub async fn cleanup_partials(dir: &Path, video_id: &str) {
    let tag = format!("[{video_id}]");
    let Ok(mut entries) = tokio::fs::read_dir(dir).await else { return };
    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.contains(&tag) && is_partial(&name) {
            let _ = tokio::fs::remove_file(entry.path()).await;
        }
    }
}

fn is_partial(name: &str) -> bool {
    if name.ends_with(".part") || name.ends_with(".ytdl") || name.contains(".temp.") {
        return true;
    }
    // "titulo [id].f137.mp4"
    name.rsplit('.')
        .nth(1)
        .is_some_and(|seg| seg.len() > 1 && seg.starts_with('f') && seg[1..].chars().all(|c| c.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn args_keep_current_command_and_add_protocol() {
        let spec = RunSpec {
            ytdlp: "yt-dlp".into(),
            ffmpeg: Some("/opt/ffmpeg".into()),
            out_dir: "/tmp/out".into(),
            profile: Profile::mp4_h264(),
            url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".into(),
        };
        let a = spec.args();
        assert_eq!(&a[..4], ["-S", "vcodec:h264,res,acodec:m4a", "--merge-output-format", "mp4"]);
        assert!(a.windows(2).any(|w| w == ["--encoding", "utf-8"]));
        assert!(a.windows(2).any(|w| w == ["--ffmpeg-location", "/opt/ffmpeg"]));
        assert!(a.windows(2).any(|w| w == ["-P", "/tmp/out"]));
        assert!(a.windows(2).any(|w| w[0] == "--progress-template" && w[1].starts_with("download:TT_PROGRESS|")));
        assert_eq!(a.last().unwrap(), "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
        assert_eq!(a[a.len() - 2], "--");
    }

    #[test]
    fn partial_detection() {
        assert!(is_partial("Video [abc].mp4.part"));
        assert!(is_partial("Video [abc].f137.mp4"));
        assert!(is_partial("Video [abc].f140.m4a.part"));
        assert!(is_partial("Video [abc].temp.mp4"));
        assert!(is_partial("Video [abc].mp4.ytdl"));
        assert!(!is_partial("Video [abc].mp4"));
        assert!(!is_partial("final.fixed.mp4"));
    }

    #[tokio::test]
    async fn cleanup_keeps_final_file() {
        let dir = tempfile::tempdir().unwrap();
        for n in ["V [abc].mp4", "V [abc].f137.mp4", "V [abc].f140.m4a.part", "W [xyz].mp4.part"] {
            std::fs::write(dir.path().join(n), b"x").unwrap();
        }
        cleanup_partials(dir.path(), "abc").await;
        let mut left: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        left.sort();
        assert_eq!(left, ["V [abc].mp4", "W [xyz].mp4.part"]);
    }
}
