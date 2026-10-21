import { useEffect, useRef } from "react";
import type { MediaKind, Profile } from "../lib/tauri";
import { hasMod, modKey } from "../lib/platform";
import { selectProfile, useSettings } from "../store/settings";
import { Icon } from "./Icon";

/** último formato usado em cada tipo, para o Vídeo|Áudio voltar a ele */
const LAST_KEY = "tt.lastProfileByKind";

function readLast(): Partial<Record<MediaKind, string>> {
  try {
    return JSON.parse(localStorage.getItem(LAST_KEY) ?? "{}");
  } catch {
    return {};
  }
}

function writeLast(kind: MediaKind, id: string) {
  try {
    localStorage.setItem(LAST_KEY, JSON.stringify({ ...readLast(), [kind]: id }));
  } catch {
    /* sem armazenamento: só não lembra */
  }
}

const KINDS: { kind: MediaKind; label: string; icon: "film" | "music-note" }[] = [
  { kind: "video", label: "Vídeo", icon: "film" },
  { kind: "audio", label: "Áudio", icon: "music-note" },
];

/**
 * §7.5 — Vídeo | Áudio + formato. A escolha vira o padrão (também usado
 * pelo atalho global e por "Colar e baixar").
 */
export function FormatPicker() {
  const profiles = useSettings((s) => s.profiles);
  const current = useSettings(selectProfile);
  const update = useSettings((s) => s.update);
  const currentRef = useRef(current);
  currentRef.current = current;

  const choose = (p: Profile) => {
    writeLast(p.kind, p.id);
    if (p.id !== currentRef.current?.id) update({ profileId: p.id });
  };

  const chooseKind = (kind: MediaKind) => {
    if (currentRef.current?.kind === kind) return;
    const options = profiles.filter((p) => p.kind === kind);
    const last = readLast()[kind];
    const next = options.find((p) => p.id === last) ?? options[0];
    if (next) choose(next);
  };

  // ⌘1 / ⌘2 (Ctrl no Windows) alternam Vídeo / Áudio (§8)
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (!hasMod(e) || e.altKey || e.shiftKey) return;
      if (e.key === "1" || e.key === "2") {
        e.preventDefault();
        chooseKind(e.key === "1" ? "video" : "audio");
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  if (!current) return null;
  const options = profiles.filter((p) => p.kind === current.kind);

  return (
    <div className="options">
      <div className="segmented" role="radiogroup" aria-label="Tipo">
        {KINDS.map(({ kind, label, icon }, i) => (
          <button
            key={kind}
            type="button"
            role="radio"
            aria-checked={current.kind === kind}
            className={current.kind === kind ? "selected" : undefined}
            title={`${label} (${modKey}${i + 1})`}
            onClick={() => chooseKind(kind)}
          >
            <Icon name={icon} />
            {label}
          </button>
        ))}
      </div>
      <label className="popup" title={current.hint}>
        <select
          aria-label="Formato"
          value={current.id}
          onChange={(e) => {
            const p = profiles.find((x) => x.id === e.target.value);
            if (p) choose(p);
          }}
        >
          {options.map((p) => (
            <option key={p.id} value={p.id}>
              {p.name}
            </option>
          ))}
        </select>
        <Icon name="chevron-updown" />
      </label>
    </div>
  );
}
