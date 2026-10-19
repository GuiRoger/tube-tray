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
