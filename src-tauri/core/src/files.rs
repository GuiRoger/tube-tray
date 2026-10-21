//! Texto e arquivos vindos do yt-dlp.
//!
//! No Windows o yt-dlp.exe escreve no código de página do sistema
//! (Windows-1252), não em UTF-8, e títulos com acento chegavam como
//! "V\u{FFFD}deos". O runner pede `--encoding utf-8`, e estas funções
//! cobrem o que ainda escapar: decodificação de reserva e localização do
//! arquivo pelo id do vídeo, que vai no nome (`Título [id].mp4`).

use std::path::{Path, PathBuf};

/// Windows-1252 nos bytes 0x80–0x9F (o resto coincide com Latin-1).
const CP1252_HIGH: [char; 32] = [
    '€', '\u{81}', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '\u{8D}', 'Ž', '\u{8F}',
    '\u{90}', '‘', '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ', '\u{9D}', 'ž', 'Ÿ',
];

/// Decodifica uma linha da saída: UTF-8 quando válido; senão Windows-1252.
pub fn decode_output(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => bytes
            .iter()
            .map(|&b| match b {
                0x80..=0x9F => CP1252_HIGH[(b - 0x80) as usize],
                _ => b as char,
            })
            .collect(),
    }
}

/// Texto que perdeu caracteres numa decodificação errada.
pub fn is_garbled(s: &str) -> bool {
    s.contains('\u{FFFD}')
}

fn is_temp(name: &str) -> bool {
    name.ends_with(".part")
        || name.ends_with(".ytdl")
        || name.contains(".temp.")
        || name
            .rsplit('.')
            .nth(1)
            .is_some_and(|seg| seg.len() > 1 && seg.starts_with('f') && seg[1..].chars().all(|c| c.is_ascii_digit()))
}

/// Procura em `dir` o arquivo final do vídeo `id`. Prefere as extensões
/// de `prefer` (na ordem dada) e, entre iguais, o mais recente. Ignora
/// temporários de downloads incompletos.
pub fn find_by_id(dir: &Path, id: &str, prefer: &[&str]) -> Option<PathBuf> {
    let tag = format!("[{id}]");
    let mut found: Vec<(usize, std::time::SystemTime, PathBuf)> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            if !name.contains(&tag) || is_temp(&name) {
                return None;
            }
            let meta = e.metadata().ok().filter(|m| m.is_file())?;
            let ext = name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
            let rank = prefer.iter().position(|p| *p == ext).unwrap_or(prefer.len());
            Some((rank, meta.modified().unwrap_or(std::time::UNIX_EPOCH), e.path()))
        })
        .collect();
    found.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)));
    found.into_iter().next().map(|(_, _, p)| p)
}

/// Nome seguro de pasta no Windows e no macOS (ex.: título da playlist).
pub fn safe_dir_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| if c.is_control() || "<>:\"/\\|?*".contains(c) { ' ' } else { c })
        .collect();
    let collapsed = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut out: String = collapsed.chars().take(80).collect();
    // o Windows não aceita nomes terminados em ponto ou espaço
    while out.ends_with(['.', ' ']) {
        out.pop();
    }
    let reserved = ["CON", "PRN", "AUX", "NUL", "COM1", "LPT1"];
    if reserved.iter().any(|r| out.eq_ignore_ascii_case(r)) {
        out.push('_');
    }
    out.trim_start_matches('.').trim().to_string()
}

/// "Título [id].mp4" → "Título" (para recuperar um título corrompido).
pub fn title_from_file(path: &Path, id: &str) -> Option<String> {
    let stem = path.file_stem()?.to_string_lossy().into_owned();
    let title = stem.strip_suffix(&format!(" [{id}]")).unwrap_or(&stem).trim().to_string();
    (!title.is_empty()).then_some(title)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_utf8_and_cp1252() {
        assert_eq!(decode_output("Vídeos ção".as_bytes()), "Vídeos ção");
        // "Vídeos – ção" em Windows-1252
        assert_eq!(decode_output(b"V\xEDdeos \x96 \xE7\xE3o"), "Vídeos – ção");
        assert!(is_garbled("V\u{FFFD}deos"));
        assert!(!is_garbled("Vídeos"));
    }

    #[test]
    fn safe_names() {
        assert_eq!(safe_dir_name("Aulas: Rust / Tokio?"), "Aulas Rust Tokio");
        assert_eq!(safe_dir_name("  Mix.. "), "Mix");
        assert_eq!(safe_dir_name("con"), "con_");
        assert_eq!(safe_dir_name("../.."), "");
        assert_eq!(safe_dir_name(&"a".repeat(200)).len(), 80);
    }

    #[test]
    fn finds_final_file_by_id() {
        let dir = tempfile::tempdir().unwrap();
        for n in ["Vídeos [abc12345678].f137.mp4", "Vídeos [abc12345678].mp4.part", "Outro [zzz12345678].mp4"] {
            std::fs::write(dir.path().join(n), b"x").unwrap();
        }
        assert_eq!(find_by_id(dir.path(), "abc12345678", &["mp4"]), None);
        std::fs::write(dir.path().join("Vídeos [abc12345678].mp4"), b"x").unwrap();
        std::fs::write(dir.path().join("Vídeos [abc12345678].mp3"), b"x").unwrap();
        let found = find_by_id(dir.path(), "abc12345678", &["mp4"]).unwrap();
        assert!(found.ends_with("Vídeos [abc12345678].mp4"));
        let audio = find_by_id(dir.path(), "abc12345678", &["mp3"]).unwrap();
        assert!(audio.ends_with("Vídeos [abc12345678].mp3"));
        assert_eq!(title_from_file(&found, "abc12345678").as_deref(), Some("Vídeos"));
    }
}
