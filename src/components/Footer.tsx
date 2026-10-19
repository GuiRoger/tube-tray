import { modKey, shortPath } from "../lib/platform";

interface Props {
  outDir: string;
  home: string | null;
  onPickFolder: () => void;
  onQuit: () => void;
}

/** §7.9 — pasta de destino (abre o seletor) e Sair. */
export function Footer({ outDir, home, onPickFolder, onQuit }: Props) {
  return (
    <footer className="footer">
      <button type="button" className="out-dir" onClick={onPickFolder} title={`${outDir}\nClique para trocar a pasta`}>
        Salvar em: {shortPath(outDir, home)}
      </button>
      <button type="button" onClick={onQuit}>
        Sair<kbd>{modKey}Q</kbd>
      </button>
    </footer>
  );
}
