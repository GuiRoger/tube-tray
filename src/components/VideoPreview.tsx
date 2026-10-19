import { useState } from "react";
import type { VideoInfo } from "../lib/tauri";
import { Icon } from "./Icon";

export type PreviewState =
  | { kind: "loading" }
  | { kind: "ready"; info: VideoInfo }
  | { kind: "error"; message: string };

/** §7.4 — card com miniatura, título e canal do vídeo colado. */
export function VideoPreview({ state }: { state: PreviewState }) {
  const [thumbFailed, setThumbFailed] = useState(false);

  if (state.kind === "loading") {
    return (
      <div className="preview" aria-busy="true" aria-label="Carregando informações do vídeo">
        <div className="preview-thumb skeleton" />
        <div className="preview-text">
          <div className="skeleton line" style={{ width: "92%" }} />
          <div className="skeleton line" style={{ width: "55%" }} />
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
          <div className="preview-title">Não foi possível carregar a prévia</div>
          <div className="preview-meta">{state.message}</div>
        </div>
      </div>
    );
  }

  const { info } = state;
  return (
    <div className="preview" title={info.title}>
      {thumbFailed ? (
        <div className="preview-thumb placeholder">
          <Icon name="film" />
        </div>
      ) : (
        <img
          className="preview-thumb"
          src={info.thumbnail}
          alt=""
          draggable={false}
          onError={() => setThumbFailed(true)}
        />
      )}
      <div className="preview-text">
        <div className="preview-title">{info.title}</div>
        {info.channel && <div className="preview-meta">{info.channel}</div>}
      </div>
    </div>
  );
}
