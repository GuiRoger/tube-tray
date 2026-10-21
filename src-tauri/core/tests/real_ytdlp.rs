//! Testes com o yt-dlp de verdade contra um servidor HTTP local.
//! Rodam só quando `TT_YTDLP` (caminho do yt-dlp) e `TT_CLIP_URL`
//! (um .mp4 servido por HTTP) estão definidos:
//!
//! ```sh
//! TT_YTDLP=./yt-dlp TT_CLIP_URL=http://127.0.0.1:8765/clip.mp4 cargo test --test real_ytdlp
//! ```

use std::time::Duration;

use tokio_util::sync::CancellationToken;
use tubetray_core::runner::{run, RunSpec};
use tubetray_core::{Profile, Stage};

fn env() -> Option<(String, String)> {
    Some((std::env::var("TT_YTDLP").ok()?, std::env::var("TT_CLIP_URL").ok()?))
}

fn spec(ytdlp: &str, url: &str, out: &std::path::Path, extra: &[&str]) -> RunSpec {
    let mut profile = Profile::mp4_h264();
    profile.args.extend(extra.iter().map(|s| s.to_string()));
    RunSpec {
        ytdlp: ytdlp.into(),
        ffmpeg: None,
        out_dir: out.to_path_buf(),
        profile,
        url: url.into(),
    }
}

#[tokio::test]
async fn downloads_and_reports_progress() {
    let Some((ytdlp, url)) = env() else { return };
    let dir = tempfile::tempdir().unwrap();
    let mut stages = Vec::new();
    let mut percents = Vec::new();
    let snap = run(&spec(&ytdlp, &url, dir.path(), &["--limit-rate", "2M"]), CancellationToken::new(), |s| {
        stages.push(s.stage);
        percents.push(s.percent);
    })
    .await;

    assert_eq!(snap.stage, Stage::Done, "erro: {:?}", snap.error);
    assert_eq!(snap.percent, 100.0);
    let title = snap.title.clone().expect("título");
    assert!(!tubetray_core::files::is_garbled(&title), "título corrompido: {title}");
    let file = snap.file.expect("caminho do arquivo final");
    assert!(std::path::Path::new(&file).is_file());
    assert_eq!(snap.size, Some(std::fs::metadata(&file).unwrap().len()));
    assert!(stages.contains(&Stage::Downloading));
    assert!(percents.windows(2).all(|w| w[1] >= w[0]), "percentual recuou: {percents:?}");
}

#[tokio::test]
async fn cancel_stops_and_cleans_up() {
    let Some((ytdlp, url)) = env() else { return };
    let dir = tempfile::tempdir().unwrap();
    let token = CancellationToken::new();
    let t = token.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(1500)).await;
        t.cancel();
    });
    let snap = tokio::time::timeout(
        Duration::from_secs(15),
        run(&spec(&ytdlp, &url, dir.path(), &["--limit-rate", "100K"]), token, |_| {}),
    )
    .await
    .expect("cancelamento deve encerrar o processo");

    assert_eq!(snap.stage, Stage::Canceled);
    let left: Vec<_> = std::fs::read_dir(dir.path()).unwrap().map(|e| e.unwrap().file_name()).collect();
    assert!(left.is_empty(), "sobraram temporários: {left:?}");
}

#[tokio::test]
async fn failure_has_message() {
    let Some((ytdlp, url)) = env() else { return };
    let dir = tempfile::tempdir().unwrap();
    let bad = format!("{}/nao-existe.mp4", &url[..url.rfind('/').unwrap()]);
    let snap = run(&spec(&ytdlp, &bad, dir.path(), &[]), CancellationToken::new(), |_| {}).await;
    assert_eq!(snap.stage, Stage::Failed);
    assert!(snap.error.is_some_and(|e| !e.is_empty()));
}

/// Perfis de áudio (e "melhor qualidade") com o ffmpeg de `TT_FFMPEG`, se
/// definido (o mesmo binário que vai no instalador, sem ffprobe).
#[tokio::test]
async fn other_profiles_produce_the_right_file() {
    let Some((ytdlp, url)) = env() else { return };
    let ffmpeg = std::env::var("TT_FFMPEG").ok().map(Into::into);
    for (profile, ext) in [
        (Profile::audio_mp3(), "mp3"),
        (Profile::audio_m4a(), "m4a"),
        (Profile::best(), "mp4"),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let spec = RunSpec {
            ytdlp: ytdlp.clone().into(),
            ffmpeg: ffmpeg.clone(),
            out_dir: dir.path().to_path_buf(),
            profile: profile.clone(),
            url: url.clone(),
        };
        let snap = run(&spec, CancellationToken::new(), |_| {}).await;
        assert_eq!(snap.stage, Stage::Done, "{}: {:?}", profile.id, snap.error);
        let file = snap.file.expect("arquivo final");
        assert!(file.ends_with(&format!(".{ext}")), "{}: {file}", profile.id);
        assert!(std::path::Path::new(&file).is_file());
        // só o arquivo final fica na pasta (o vídeo original some no -x)
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1, "{}", profile.id);
    }
}
