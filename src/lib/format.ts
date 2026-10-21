// Formatação pt-BR para velocidade, tempo e reconhecimento de links.

const nf1 = new Intl.NumberFormat("pt-BR", { maximumFractionDigits: 1, minimumFractionDigits: 1 });
const nf0 = new Intl.NumberFormat("pt-BR", { maximumFractionDigits: 0 });

export function formatSpeed(bytesPerSec: number | null): string {
  if (bytesPerSec == null || !isFinite(bytesPerSec) || bytesPerSec <= 0) return "";
  const kb = bytesPerSec / 1024;
  if (kb < 1024) return `${nf0.format(kb)} KB/s`;
  return `${nf1.format(kb / 1024)} MB/s`;
}

export function formatEta(seconds: number | null): string {
  if (seconds == null || !isFinite(seconds) || seconds < 0) return "";
  const s = Math.round(seconds);
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const ss = String(s % 60).padStart(2, "0");
  return h > 0 ? `${h}:${String(m).padStart(2, "0")}:${ss}` : `${m}:${ss}`;
}

export function formatPercent(p: number): string {
  return `${Math.floor(Math.min(100, Math.max(0, p)))}%`;
}

/** Checagem rápida no front; a validação real fica no núcleo Rust. */
export function looksLikeYoutube(text: string): boolean {
  return /(^|\/\/|\.)(youtube\.com|youtu\.be)\//i.test(text.trim()) || /^(www\.|m\.)?youtu/i.test(text.trim());
}

/** Nome do arquivo a partir do caminho completo (Windows ou Unix). */
export function fileName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

/** "18s restantes", "2 min restantes", "1 h 5 min restantes" */
export function formatRemaining(seconds: number | null): string {
  if (seconds == null || !isFinite(seconds) || seconds < 0) return "";
  const s = Math.round(seconds);
  if (s < 60) return `${s}s restantes`;
  if (s < 3600) return `${Math.round(s / 60)} min restantes`;
  const h = Math.floor(s / 3600);
  const m = Math.round((s % 3600) / 60);
  return m ? `${h} h ${m} min restantes` : `${h} h restantes`;
}

const nfSize = new Intl.NumberFormat("pt-BR", { maximumFractionDigits: 1 });

/** 186 MB, 1,2 GB */
export function formatSize(bytes: number | null | undefined): string {
  if (bytes == null || !isFinite(bytes) || bytes <= 0) return "";
  const mb = bytes / (1024 * 1024);
  if (mb < 1) return `${Math.max(1, Math.round(bytes / 1024))} KB`;
  if (mb < 1024) return `${Math.round(mb)} MB`;
  return `${nfSize.format(mb / 1024)} GB`;
}

/** Id da playlist (`list=`) do link, se houver. */
export function playlistParam(text: string): string | null {
  const m = /[?&]list=([\w-]{2,64})/.exec(text.trim());
  return m ? m[1] : null;
}

/** O link aponta para um vídeo (não só para uma playlist). */
export function hasVideo(text: string): boolean {
  const t = text.trim();
  return /[?&]v=[\w-]{11}/.test(t) || /(youtu\.be|\/shorts|\/live|\/embed)\/[\w-]{11}/.test(t);
}

/** Duração de vídeo: "4:05", "1:02:03". */
export function formatDuration(seconds: number | null | undefined): string {
  if (seconds == null || !isFinite(seconds) || seconds <= 0) return "";
  return formatEta(seconds);
}
