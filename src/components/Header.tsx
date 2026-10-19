import appIcon from "../assets/app-icon-64.png";
import { modKey } from "../lib/platform";
import { Icon } from "./Icon";

interface Props {
  title: string;
  onOpenFolder: () => void;
  onSettings: () => void;
  settingsActive?: boolean;
}

/** §7.2 — mini ícone, título e ações Abrir pasta / Ajustes. */
export function Header({ title, onOpenFolder, onSettings, settingsActive }: Props) {
  return (
    <header className="header" data-tauri-drag-region>
      <img className="app-icon" src={appIcon} alt="" draggable={false} />
      <h1 data-tauri-drag-region>{title}</h1>
      <button
        type="button"
        className="icon-button"
        onClick={onOpenFolder}
        aria-label="Abrir pasta"
        title={`Abrir pasta (${modKey}O)`}
      >
        <Icon name="folder" />
      </button>
      <button
        type="button"
        className="icon-button"
        onClick={onSettings}
        aria-label={settingsActive ? "Voltar" : "Ajustes"}
        aria-pressed={settingsActive}
        title={settingsActive ? "Voltar (Esc)" : `Ajustes (${modKey},)`}
        style={settingsActive ? { color: "var(--accent)" } : undefined}
      >
        <Icon name="gear" />
      </button>
    </header>
  );
}
