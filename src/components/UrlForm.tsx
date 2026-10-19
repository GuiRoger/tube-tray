import { forwardRef, useEffect, useImperativeHandle, useRef, useState } from "react";
import { readText } from "@tauri-apps/plugin-clipboard-manager";
import { api, type VideoInfo } from "../lib/tauri";
import { looksLikeYoutube } from "../lib/format";
import { modKey } from "../lib/platform";
import { Icon } from "./Icon";
import { VideoPreview, type PreviewState } from "./VideoPreview";

/** prévias já buscadas nesta sessão (por link) */
const previewCache = new Map<string, VideoInfo>();

export interface UrlFormHandle {
  /** Preenche com o link da área de transferência (se for do YouTube). */
  prefillFromClipboard: () => Promise<void>;
  focus: () => void;
}

const INVALID = "Esse link não parece ser do YouTube.";

/** §7.3 campo de URL + §7.6 botão primário. */
export const UrlForm = forwardRef<UrlFormHandle>(function UrlForm(_props, ref) {
  const [url, setUrl] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [touched, setTouched] = useState(false);
  const [busy, setBusy] = useState(false);
  const input = useRef<HTMLInputElement>(null);
  /** último valor preenchido automaticamente: só ele pode ser trocado sem perguntar */
  const autoFilled = useRef<string | null>(null);

  const [preview, setPreview] = useState<PreviewState | null>(null);

  const trimmed = url.trim();
  const valid = looksLikeYoutube(trimmed);

  // prévia (título + miniatura) quando há um link válido no campo
  useEffect(() => {
    if (!valid) {
      setPreview(null);
      return;
    }
    const cached = previewCache.get(trimmed);
    if (cached) {
      setPreview({ kind: "ready", info: cached });
      return;
    }
    let stale = false;
    setPreview({ kind: "loading" });
    const timer = setTimeout(() => {
      api.videoInfo(trimmed).then(
        (info) => {
          previewCache.set(trimmed, info);
          if (!stale) setPreview({ kind: "ready", info });
        },
        (err) => !stale && setPreview({ kind: "error", message: String(err) }),
      );
    }, 250);
    return () => {
      stale = true;
      clearTimeout(timer);
    };
  }, [trimmed, valid]);
  const shownError = error ?? (touched && trimmed && !valid ? INVALID : null);

  const fill = (text: string) => {
    setUrl(text);
    setError(null);
    setTouched(true);
    requestAnimationFrame(() => {
      input.current?.focus();
      input.current?.select();
    });
  };

  useImperativeHandle(ref, () => ({
    prefillFromClipboard: async () => {
      const current = input.current?.value.trim() ?? "";
      if (current && current !== autoFilled.current) return; // não apaga o que o usuário digitou
      try {
        const text = (await readText())?.trim() ?? "";
        if (text && looksLikeYoutube(text)) {
          autoFilled.current = text;
          fill(text);
          return;
        }
      } catch {
        /* área de transferência vazia ou sem texto */
      }
      input.current?.focus();
    },
    focus: () => input.current?.focus(),
  }));

  const paste = async () => {
    try {
      const text = (await readText())?.trim() ?? "";
      if (text) fill(text);
    } catch {
      input.current?.focus();
    }
  };

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setTouched(true);
    if (!valid || busy) return;
    setBusy(true);
    try {
      await api.enqueue(trimmed, preview?.kind === "ready" ? preview.info.title : null);
      setUrl("");
      setTouched(false);
      setError(null);
      autoFilled.current = null;
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
      input.current?.focus();
    }
  };

  return (
    <form onSubmit={submit} style={{ display: "contents" }}>
      <label className="url-field">
        <Icon name="link" />
        <input
          ref={input}
          type="text"
          inputMode="url"
          spellCheck={false}
          autoComplete="off"
          placeholder="Cole o link do YouTube…"
          aria-label="Link do vídeo"
          aria-invalid={!!shownError}
          value={url}
          onChange={(e) => {
            setUrl(e.target.value);
            setError(null);
          }}
          onBlur={() => setTouched(true)}
          onPaste={() => setTouched(true)}
        />
        <button type="button" className="keycap" onClick={paste} title="Colar" aria-label="Colar link">
          {modKey}V
        </button>
      </label>

      {preview && !shownError && <VideoPreview key={trimmed} state={preview} />}

      {shownError && (
        <div className="field-message" role="alert">
          <Icon name="exclamation" />
          <span>{shownError}</span>
        </div>
      )}

      <button type="submit" className="primary-button" disabled={!valid || busy}>
        {busy ? (
          <>
            <span className="spinner" aria-hidden="true" />
            Preparando…
          </>
        ) : (
          <>
            <Icon name="download" />
            Baixar
          </>
        )}
      </button>
    </form>
  );
});
