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
