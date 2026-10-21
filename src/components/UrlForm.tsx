import { forwardRef, useEffect, useImperativeHandle, useRef, useState } from "react";
import { readText } from "@tauri-apps/plugin-clipboard-manager";
import { api, type PlaylistInfo, type VideoInfo } from "../lib/tauri";
import { hasVideo, looksLikeYoutube, playlistParam } from "../lib/format";
import { modKey } from "../lib/platform";
import { selectProfile, useSettings } from "../store/settings";
import { FormatPicker } from "./FormatPicker";
import { Icon } from "./Icon";
import { PlaylistPicker, type PlaylistState } from "./PlaylistPicker";
import { VideoPreview, type PreviewState } from "./VideoPreview";

/** prévias já buscadas nesta sessão (por link) */
const previewCache = new Map<string, VideoInfo>();
/** playlists já lidas nesta sessão (por id + link de mix) */
const playlistCache = new Map<string, PlaylistInfo>();

const allAvailable = (info: PlaylistInfo) => new Set(info.entries.filter((e) => e.available).map((e) => e.id));

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
  const [playlist, setPlaylist] = useState<PlaylistState | null>(null);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [notice, setNotice] = useState<string | null>(null);
  const profileId = useSettings((s) => selectProfile(s)?.id ?? null);

  const trimmed = url.trim();
  const valid = looksLikeYoutube(trimmed);
  const listId = valid ? playlistParam(trimmed) : null;
  const isVideo = valid && hasVideo(trimmed);
  // link só de playlist abre direto a escolha; vídeo dentro de playlist pergunta
  const [wantList, setWantList] = useState(false);
  useEffect(() => setWantList(!!listId && !isVideo), [listId, isVideo, trimmed]);
  const playlistMode = !!listId && (wantList || !isVideo);

  // vídeos da playlist, quando o usuário quer escolher os itens
  useEffect(() => {
    if (!playlistMode) {
      setPlaylist(null);
      return;
    }
    const key = listId!.startsWith("RD") ? trimmed : listId!;
    const cached = playlistCache.get(key);
    if (cached) {
      setPlaylist({ kind: "ready", info: cached });
      setSelected(allAvailable(cached));
      return;
    }
    let stale = false;
    setPlaylist({ kind: "loading" });
    api.playlistInfo(trimmed).then(
      (info) => {
        playlistCache.set(key, info);
        if (stale) return;
        setPlaylist({ kind: "ready", info });
        setSelected(allAvailable(info));
      },
      (err) => !stale && setPlaylist({ kind: "error", message: String(err) }),
    );
    return () => {
      stale = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [playlistMode, listId]);

  // prévia (título + miniatura) quando há um link de vídeo no campo
  useEffect(() => {
    if (!valid || !isVideo || playlistMode) {
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
  }, [trimmed, valid, isVideo, playlistMode]);
  const shownError = error ?? (touched && trimmed && !valid ? INVALID : null);

  const chosen =
    playlist?.kind === "ready" ? playlist.info.entries.filter((e) => e.available && selected.has(e.id)) : [];
  const canSubmit = valid && !busy && (playlistMode ? chosen.length > 0 : isVideo);

  const fill = (text: string) => {
    setUrl(text);
    setError(null);
    setNotice(null);
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
    if (!canSubmit) return;
    setBusy(true);
    try {
      if (playlistMode && playlist?.kind === "ready") {
        const added = await api.enqueueMany(
          chosen.map((e) => ({ url: e.url, title: e.title })),
          profileId,
          playlist.info.title,
        );
        setNotice(`${added === 1 ? "1 vídeo entrou" : `${added} vídeos entraram`} na fila.`);
      } else {
        await api.enqueue(trimmed, preview?.kind === "ready" ? preview.info.title : null, profileId);
        setNotice(null);
      }
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
            setNotice(null);
          }}
          onBlur={() => setTouched(true)}
          onPaste={() => setTouched(true)}
        />
        <button type="button" className="keycap" onClick={paste} title="Colar" aria-label="Colar link">
          {modKey}V
        </button>
      </label>

      {preview && !shownError && !playlistMode && <VideoPreview key={trimmed} state={preview} />}

      {listId && isVideo && !playlistMode && !shownError && (
        <div className="playlist-offer">
          <span>Faz parte de uma playlist ·</span>
          <button type="button" className="link-button" onClick={() => setWantList(true)}>
            Escolher vídeos
          </button>
        </div>
      )}

      {playlist && !shownError && (
        <PlaylistPicker
          state={playlist}
          selected={selected}
          onToggle={(id) =>
            setSelected((prev) => {
              const next = new Set(prev);
              if (next.has(id)) next.delete(id);
              else next.add(id);
              return next;
            })
          }
          onToggleAll={(all) =>
            setSelected(all && playlist.kind === "ready" ? allAvailable(playlist.info) : new Set())
          }
          onBack={isVideo ? () => setWantList(false) : undefined}
        />
      )}

      {shownError && (
        <div className="field-message" role="alert">
          <Icon name="exclamation" />
          <span>{shownError}</span>
        </div>
      )}

      {notice && !trimmed && (
        <div className="field-message ok" role="status">
          <Icon name="check-circle" />
          <span>{notice}</span>
        </div>
      )}

      <FormatPicker />

      <button type="submit" className="primary-button" disabled={!canSubmit}>
        {busy ? (
          <>
            <span className="spinner" aria-hidden="true" />
            Preparando…
          </>
        ) : (
          <>
            <Icon name="download" />
            {playlistMode && chosen.length > 0
              ? chosen.length === 1
                ? "Baixar 1 vídeo"
                : `Baixar ${chosen.length} vídeos`
              : "Baixar"}
          </>
        )}
      </button>
    </form>
  );
});
