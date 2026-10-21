import { useState } from "react";
import type { PlaylistEntry, PlaylistInfo } from "../lib/tauri";
import { formatDuration } from "../lib/format";
import { useJobs } from "../store/jobs";
import { Icon } from "./Icon";

export type PlaylistState =
  | { kind: "loading" }
  | { kind: "ready"; info: PlaylistInfo }
  | { kind: "error"; message: string };

const videos = (n: number) => (n === 1 ? "1 vídeo" : `${n} vídeos`);

function Thumb({ entry }: { entry: PlaylistEntry }) {
  const [failed, setFailed] = useState(false);
  if (failed) {
    return (
      <span className="pl-thumb placeholder">
        <Icon name="film" />
      </span>
    );
  }
  return (
    <img
      className="pl-thumb"
      src={entry.thumbnail}
      alt=""
      loading="lazy"
      draggable={false}
      onError={() => setFailed(true)}
    />
  );
}

interface Props {
  state: PlaylistState;
  selected: Set<string>;
  onToggle: (id: string) => void;
  onToggleAll: (all: boolean) => void;
  /** veio de um link de vídeo: volta para "só este vídeo" */
  onBack?: () => void;
}

/** §7.4 (playlist) — card "Playlist · N vídeos" + lista para escolher os itens. */
export function PlaylistPicker({ state, selected, onToggle, onToggleAll, onBack }: Props) {
  const ytdlpInstall = useJobs((s) => s.ytdlpInstall);

  if (state.kind === "loading") {
    return (
      <div className="preview" aria-busy="true" aria-label="Carregando a playlist">
        <div className="preview-thumb skeleton" />
        <div className="preview-text">
          <div className="skeleton line" style={{ width: "80%" }} />
          <div className="preview-meta">
            {ytdlpInstall != null
              ? `Instalando o yt-dlp… ${Math.floor(ytdlpInstall * 100)}%`
              : "Lendo a playlist…"}
          </div>
        </div>
      </div>
    );
  }

  if (state.kind === "error") {
    return (
      <div className="preview preview-error" role="status">
        <div className="preview-thumb placeholder">
          <Icon name="exclamation" />
        </div>
        <div className="preview-text">
          <div className="preview-title">Não foi possível abrir a playlist</div>
          <div className="preview-meta" title={state.message}>
            {state.message}
          </div>
          {onBack && (
            <button type="button" className="link-button" onClick={onBack}>
              Baixar só este vídeo
            </button>
          )}
        </div>
      </div>
    );
  }

  const { info } = state;
  const available = info.entries.filter((e) => e.available);
  const allChecked = available.length > 0 && available.every((e) => selected.has(e.id));
  const first = available[0] ?? info.entries[0];

  return (
    <div className="playlist">
      <div className="preview" title={info.title}>
        {first ? <Thumb entry={first} /> : null}
        <div className="preview-text">
          <div className="preview-title">{info.title}</div>
          <div className="preview-meta">
            {["Playlist", videos(info.entries.length) + (info.truncated ? "+" : ""), info.channel]
              .filter(Boolean)
              .join(" · ")}
          </div>
        </div>
      </div>

      <div className="pl-toolbar">
        <label className="pl-check">
          <input
            type="checkbox"
            checked={allChecked}
            ref={(el) => {
              if (el) el.indeterminate = !allChecked && selected.size > 0;
            }}
            onChange={(e) => onToggleAll(e.target.checked)}
          />
          <span>
            {selected.size} de {available.length} selecionados
          </span>
        </label>
        {onBack && (
          <button type="button" className="link-button" onClick={onBack}>
            Só este vídeo
          </button>
        )}
      </div>

      <ul className="pl-list" aria-label="Vídeos da playlist">
        {info.entries.map((e, i) => (
          <li key={e.id} className={e.available ? undefined : "unavailable"}>
            <label className="pl-item" title={e.title}>
              <input
                type="checkbox"
                checked={selected.has(e.id)}
                disabled={!e.available}
                onChange={() => onToggle(e.id)}
              />
              <span className="pl-index">{i + 1}</span>
              <Thumb entry={e} />
              <span className="pl-title">{e.available ? e.title : "Vídeo indisponível"}</span>
              <span className="pl-duration">{formatDuration(e.duration)}</span>
            </label>
          </li>
        ))}
      </ul>
      {info.truncated && <small className="pl-note">Mostrando os primeiros {info.entries.length} vídeos.</small>}
    </div>
  );
}
