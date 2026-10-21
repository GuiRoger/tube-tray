// Diferenças de texto e atalhos entre macOS e Windows.
export const isMac = /Mac/i.test(navigator.platform) || /Macintosh/i.test(navigator.userAgent);

/** Rótulo do modificador: ⌘ no Mac, Ctrl no Windows. */
export const modKey = isMac ? "⌘" : "Ctrl+";

export const revealLabel = isMac ? "Mostrar no Finder" : "Mostrar na pasta";

/** true quando o atalho usa ⌘ (Mac) ou Ctrl (Windows). */
export const hasMod = (e: KeyboardEvent | React.KeyboardEvent) => (isMac ? e.metaKey : e.ctrlKey);

/** Encurta o caminho trocando a pasta do usuário por "~". */
export function shortPath(path: string, home: string | null): string {
  if (!home) return path;
  const h = home.replace(/[\\/]+$/, "");
  if (path.toLowerCase().startsWith(h.toLowerCase())) {
    const rest = path.slice(h.length).replace(/\\/g, "/");
    return `~${rest.startsWith("/") ? rest : `/${rest}`}`;
  }
  return path;
}

/** "CommandOrControl+Alt+Y" → "⌥⌘Y" no Mac, "Ctrl+Alt+Y" no Windows. */
export function shortcutLabel(accel: string): string {
  const parts = accel.split("+").map((p) => p.trim()).filter(Boolean);
  const key = parts.pop() ?? "";
  const mods = new Set(parts.map((p) => p.toLowerCase()));
  const has = (...names: string[]) => names.some((n) => mods.has(n));
  const k = key.replace(/^Key([A-Z])$/i, "$1").replace(/^Digit(\d)$/i, "$1");
  if (isMac) {
    return (
      (has("control", "ctrl") ? "⌃" : "") +
      (has("alt", "option") ? "⌥" : "") +
      (has("shift") ? "⇧" : "") +
      (has("commandorcontrol", "cmdorctrl", "command", "cmd", "super", "meta") ? "⌘" : "") +
      k.toUpperCase()
    );
  }
  const out: string[] = [];
  if (has("commandorcontrol", "cmdorctrl", "control", "ctrl")) out.push("Ctrl");
  if (has("alt")) out.push("Alt");
  if (has("shift")) out.push("Shift");
  if (has("super", "meta", "command", "cmd")) out.push("Win");
  out.push(k.toUpperCase());
  return out.join("+");
}

const MODIFIER_CODES = /^(Shift|Control|Alt|Meta|OS)(Left|Right)?$/;

/**
 * Converte uma tecla pressionada no formato de atalho do Tauri.
 * `null` enquanto só há modificadores, ou sem Ctrl/⌘/Alt (atalho fraco).
 */
export function shortcutFromEvent(e: KeyboardEvent | React.KeyboardEvent): string | null {
  if (MODIFIER_CODES.test(e.code) || !e.code) return null;
  const primary = isMac ? e.metaKey : e.ctrlKey;
  const mods: string[] = [];
  if (primary) mods.push("CommandOrControl");
  if (isMac && e.ctrlKey) mods.push("Control");
  if (!isMac && e.metaKey) mods.push("Super");
  if (e.altKey) mods.push("Alt");
  if (e.shiftKey) mods.push("Shift");
  if (!primary && !e.altKey && !e.ctrlKey) return null;
  const key = e.code.replace(/^Key([A-Z])$/, "$1").replace(/^Digit(\d)$/, "$1");
  return [...mods, key].join("+");
}
