//! Protocolo de linhas entre o yt-dlp e o TubeTray.
//!
//! Em vez de interpretar a saída "humana" do yt-dlp (que muda entre versões),
//! pedimos linhas próprias com `--progress-template` e `--print`, todas
//! prefixadas com `TT_`. O resto da saída só é usado para erros e merge.

use serde::{Deserialize, Serialize};

pub const PROGRESS_TEMPLATE: &str = "download:TT_PROGRESS|%(progress.downloaded_bytes)s|%(progress.total_bytes)s|%(progress.total_bytes_estimate)s|%(progress.speed)s|%(progress.eta)s";
pub const META_TEMPLATE: &str = "before_dl:TT_META|%(id)s|%(format_id)s|%(title)s";
pub const FILE_TEMPLATE: &str = "after_move:TT_FILE|%(filepath)s";

/// Um evento extraído de uma linha da saída do yt-dlp.
#[derive(Debug, Clone, PartialEq)]
pub enum LineEvent {
    Meta { id: String, title: String, parts: u32 },
    Progress { downloaded: f64, total: Option<f64>, speed: Option<f64>, eta: Option<u64> },
    Merging,
    File(String),
    Error(String),
}

fn num(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.is_empty() || s.eq_ignore_ascii_case("na") || s.eq_ignore_ascii_case("none") {
        return None;
    }
    s.parse::<f64>().ok().filter(|v| v.is_finite() && *v >= 0.0)
}

pub fn parse_line(line: &str) -> Option<LineEvent> {
    let line = line.trim_end_matches(['\r', '\n']).trim_start();
    if let Some(rest) = line.strip_prefix("TT_PROGRESS|") {
        let f: Vec<&str> = rest.split('|').collect();
        if f.len() < 5 {
            return None;
        }
        return Some(LineEvent::Progress {
            downloaded: num(f[0])?,
            total: num(f[1]).or_else(|| num(f[2])),
            speed: num(f[3]),
            eta: num(f[4]).map(|v| v as u64),
        });
    }
    if let Some(rest) = line.strip_prefix("TT_META|") {
        // o título vem por último porque pode conter '|'
        let mut f = rest.splitn(3, '|');
        let id = f.next()?.trim().to_string();
        let format_id = f.next().unwrap_or("");
        let title = f.next().unwrap_or("").trim().to_string();
        let parts = format_id.split('+').filter(|p| !p.is_empty()).count().max(1) as u32;
        return Some(LineEvent::Meta { id, title, parts });
    }
    if let Some(path) = line.strip_prefix("TT_FILE|") {
        return Some(LineEvent::File(path.trim().to_string()));
    }
    if line.starts_with("[Merger]") {
        return Some(LineEvent::Merging);
    }
    if let Some(msg) = line.strip_prefix("ERROR:") {
        return Some(LineEvent::Error(msg.trim().to_string()));
    }
    None
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Stage {
    Queued,
    Starting,
    Downloading,
    Merging,
    Done,
    Failed,
    Canceled,
}

/// Fotografia do progresso de um download, pronta para a UI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub stage: Stage,
    pub title: Option<String>,
    /// 0–100, considerando todas as partes (vídeo + áudio).
    pub percent: f64,
    /// bytes por segundo
    pub speed: Option<f64>,
    /// segundos restantes da parte atual
    pub eta: Option<u64>,
    pub file: Option<String>,
    /// tamanho do arquivo final em bytes (quando concluído)
    pub size: Option<u64>,
    /// mensagem amigável quando `stage == Failed`
    pub error: Option<String>,
    /// concluído, mas o arquivo foi excluído do disco
    #[serde(default)]
    pub missing: bool,
}

/// Junta os eventos de linha num estado único.
/// O yt-dlp baixa vídeo e áudio separados; o percentual de cada parte volta
/// a zero, então o tracker soma as partes para mostrar uma barra contínua.
#[derive(Debug, Clone)]
pub struct Tracker {
    snap: Snapshot,
    parts: u32,
    part: u32,
    last_downloaded: f64,
    pub video_id: Option<String>,
    pub last_error: Option<String>,
}

impl Default for Tracker {
    fn default() -> Self {
        Self {
            snap: Snapshot {
                stage: Stage::Starting,
                title: None,
                percent: 0.0,
                speed: None,
                eta: None,
                file: None,
                size: None,
                error: None,
                missing: false,
            },
            parts: 1,
            part: 0,
            last_downloaded: 0.0,
            video_id: None,
            last_error: None,
        }
    }
}

impl Tracker {
    pub fn snapshot(&self) -> &Snapshot {
        &self.snap
    }

    /// Aplica um evento. Retorna `true` se algo visível mudou.
    pub fn apply(&mut self, ev: LineEvent) -> bool {
        match ev {
            LineEvent::Meta { id, title, parts } => {
                if !id.is_empty() {
                    self.video_id = Some(id);
                }
                if !title.is_empty() {
                    self.snap.title = Some(title);
                }
                self.parts = parts.max(1);
                true
            }
            LineEvent::Progress { downloaded, total, speed, eta } => {
                if downloaded + 1.0 < self.last_downloaded && self.part + 1 < self.parts {
                    self.part += 1;
                }
                self.last_downloaded = downloaded;
                let frac = match total {
                    Some(t) if t > 0.0 => (downloaded / t).clamp(0.0, 1.0),
                    _ => 0.0,
                };
                let overall = (self.part as f64 + frac) / self.parts as f64 * 100.0;
                self.snap.percent = overall.max(self.snap.percent).min(100.0);
                // última parte completa: o yt-dlp agora junta vídeo e áudio
                self.snap.stage = if self.snap.percent >= 100.0 { Stage::Merging } else { Stage::Downloading };
                self.snap.speed = speed;
                self.snap.eta = eta;
                true
            }
            LineEvent::Merging => {
                self.snap.stage = Stage::Merging;
                self.snap.percent = 100.0;
                self.snap.speed = None;
                self.snap.eta = None;
                true
            }
            LineEvent::File(path) => {
                self.snap.file = Some(path);
                false
            }
            LineEvent::Error(msg) => {
                self.last_error = Some(msg);
                false
            }
        }
    }

    pub fn set_file(&mut self, file: Option<String>) {
        self.snap.file = file;
    }

    pub fn set_title(&mut self, title: Option<String>) {
        self.snap.title = title;
    }

    pub fn set_size(&mut self, size: Option<u64>) {
        self.snap.size = size;
    }

    pub fn finish(&mut self, stage: Stage, error: Option<String>) {
        self.snap.stage = stage;
        self.snap.error = error;
        self.snap.speed = None;
        self.snap.eta = None;
        if stage == Stage::Done {
            self.snap.percent = 100.0;
        }
    }
}

/// Traduz os erros mais comuns do yt-dlp para mensagens curtas em português.
pub fn friendly_error(raw: &str) -> String {
    let l = raw.to_ascii_lowercase();
    let msg = if l.contains("private video") {
        "Este vídeo é privado."
    } else if l.contains("confirm your age") || l.contains("age-restricted") || l.contains("inappropriate for some users") {
        "Vídeo com restrição de idade; o YouTube exige login."
    } else if l.contains("members-only") || l.contains("join this channel") {
        "Vídeo exclusivo para membros do canal."
    } else if l.contains("video unavailable") || l.contains("has been removed") || l.contains("not available") {
        "Vídeo indisponível ou removido."
    } else if l.contains("sign in to confirm you") && l.contains("bot") {
        "O YouTube pediu verificação anti-bot. Tente atualizar o yt-dlp."
    } else if l.contains("http error 429") {
        "Muitas requisições ao YouTube. Aguarde alguns minutos."
    } else if l.contains("unable to download webpage")
        || l.contains("getaddrinfo")
        || l.contains("name or service not known")
        || l.contains("timed out")
        || l.contains("connection")
    {
        "Sem conexão com o YouTube. Verifique a internet."
    } else if l.contains("ffmpeg") {
        "Falha ao juntar vídeo e áudio (ffmpeg)."
    } else if l.contains("requested format is not available") {
        "Formato MP4 (H.264) indisponível para este vídeo."
    } else if l.contains("no space left") {
        "Sem espaço em disco na pasta de destino."
    } else {
        return raw.trim().to_string();
    };
    msg.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_progress_with_na_fields() {
        assert_eq!(
            parse_line("TT_PROGRESS|1048576|NA|4194304.5|524288.0|6\n"),
            Some(LineEvent::Progress {
                downloaded: 1048576.0,
                total: Some(4194304.5),
                speed: Some(524288.0),
                eta: Some(6)
            })
        );
        assert_eq!(
            parse_line("TT_PROGRESS|10|NA|NA|NA|NA"),
            Some(LineEvent::Progress { downloaded: 10.0, total: None, speed: None, eta: None })
        );
    }

    #[test]
    fn parses_meta_with_pipe_in_title() {
        assert_eq!(
            parse_line("TT_META|dQw4w9WgXcQ|137+140|Rick | Never Gonna"),
            Some(LineEvent::Meta { id: "dQw4w9WgXcQ".into(), title: "Rick | Never Gonna".into(), parts: 2 })
        );
        assert_eq!(
            parse_line("TT_META|abc|18|Single"),
            Some(LineEvent::Meta { id: "abc".into(), title: "Single".into(), parts: 1 })
        );
    }

    #[test]
    fn parses_other_lines() {
        assert_eq!(
            parse_line("[Merger] Merging formats into \"x.mp4\""),
            Some(LineEvent::Merging)
        );
        assert_eq!(
            parse_line("TT_FILE|C:\\Users\\gui\\Downloads\\x.mp4\r"),
            Some(LineEvent::File("C:\\Users\\gui\\Downloads\\x.mp4".into()))
        );
        assert_eq!(
            parse_line("ERROR: [youtube] abc: Private video. Sign in"),
            Some(LineEvent::Error("[youtube] abc: Private video. Sign in".into()))
        );
        assert_eq!(parse_line("[youtube] Extracting URL"), None);
    }

    #[test]
    fn tracker_spans_two_parts() {
        let mut t = Tracker::default();
        t.apply(LineEvent::Meta { id: "abc".into(), title: "X".into(), parts: 2 });
        let p = |d: f64, tot: f64| LineEvent::Progress { downloaded: d, total: Some(tot), speed: Some(1.0), eta: Some(1) };
        t.apply(p(50.0, 100.0));
        assert_eq!(t.snapshot().percent, 25.0);
        t.apply(p(100.0, 100.0));
        assert_eq!(t.snapshot().percent, 50.0);
        // segunda parte (áudio) recomeça do zero
        t.apply(p(10.0, 40.0));
        assert_eq!(t.snapshot().percent, 62.5);
        t.apply(p(40.0, 40.0));
        assert_eq!(t.snapshot().percent, 100.0);
        assert_eq!(t.snapshot().stage, Stage::Merging);
        t.apply(LineEvent::Merging);
        assert_eq!(t.snapshot().stage, Stage::Merging);
        t.finish(Stage::Done, None);
        assert_eq!(t.snapshot().stage, Stage::Done);
        assert_eq!(t.snapshot().speed, None);
    }

    #[test]
    fn percent_never_goes_backwards() {
        let mut t = Tracker::default();
        let p = |d: f64| LineEvent::Progress { downloaded: d, total: Some(100.0), speed: None, eta: None };
        t.apply(p(60.0));
        t.apply(p(20.0)); // parts = 1: não avança parte, mas barra não recua
        assert_eq!(t.snapshot().percent, 60.0);
    }

    #[test]
    fn friendly_errors() {
        assert_eq!(friendly_error("[youtube] x: Private video. Sign in if you've been granted access"), "Este vídeo é privado.");
        assert_eq!(friendly_error("Unable to download webpage: <urlopen error [Errno 11001] getaddrinfo failed>"), "Sem conexão com o YouTube. Verifique a internet.");
        assert_eq!(friendly_error("algo novo"), "algo novo");
    }
}
