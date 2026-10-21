import { useEffect, useState } from "react";
import { api } from "../lib/tauri";
import { shortcutFromEvent, shortcutLabel } from "../lib/platform";

interface Props {
  value: string;
  disabled?: boolean;
  onChange: (accel: string) => void;
}

/** Botão que grava um atalho: clique, pressione as teclas (Esc cancela). */
export function ShortcutField({ value, disabled, onChange }: Props) {
  const [recording, setRecording] = useState(false);

  useEffect(() => {
    if (!recording) return;
    api.pauseShortcut(true).catch(() => {});
    const onKey = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      if (e.key === "Escape") {
        setRecording(false);
        return;
      }
      const accel = shortcutFromEvent(e);
      if (accel) {
        setRecording(false);
        onChange(accel);
      }
    };
    // captura antes dos atalhos da janela (Esc fecharia os Ajustes)
    window.addEventListener("keydown", onKey, true);
    return () => {
      window.removeEventListener("keydown", onKey, true);
      api.pauseShortcut(false).catch(() => {});
    };
  }, [recording, onChange]);

  return (
    <button
      type="button"
      className={`keycap-button${recording ? " recording" : ""}`}
      disabled={disabled}
      onClick={() => setRecording((r) => !r)}
      onBlur={() => setRecording(false)}
      title={recording ? "Pressione as teclas (Esc cancela)" : "Clique para trocar o atalho"}
      aria-label={recording ? "Gravando atalho" : `Atalho ${shortcutLabel(value)}, clique para trocar`}
    >
      {recording ? "Pressione as teclas…" : shortcutLabel(value)}
    </button>
  );
}
