import { useCallback, useEffect, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { openUrl } from "@tauri-apps/plugin-opener";
import { api } from "../lib/tauri";
import { links, support } from "../lib/support";
import { shortPath } from "../lib/platform";
import { useJobs, selectActiveCount } from "../store/jobs";
import { useSettings } from "../store/settings";
import { Icon } from "./Icon";
import { ShortcutField } from "./ShortcutField";

const PARALLEL = [1, 2, 3, 4, 5];

interface Props {
  home: string | null;
  onPickFolder: () => void;
  onOpenLicenses: () => void;
}

function Toggle({
  label,
  hint,
  checked,
  onChange,
}: {
  label: string;
  hint?: string;
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <label className="group-row">
      <span className="label">
        <span>{label}</span>
        {hint && <small>{hint}</small>}
      </span>
      <input type="checkbox" role="switch" className="switch" checked={checked} onChange={(e) => onChange(e.target.checked)} />
    </label>
  );
}

/** Ajustes (§7.11, versão compacta dentro do popover). */
export function SettingsView({ home, onPickFolder, onOpenLicenses }: Props) {
  const { settings, update, error, profiles } = useSettings();
  const [shortcutError, setShortcutError] = useState<string | null>(null);
  const activeCount = useJobs(selectActiveCount);
  const [version, setVersion] = useState<string | null | undefined>(undefined);
  const [updating, setUpdating] = useState(false);
  const [updateMsg, setUpdateMsg] = useState<{ text: string; error: boolean } | null>(null);

  const [appVersion, setAppVersion] = useState("");
  const [pixCopied, setPixCopied] = useState(false);

  useEffect(() => {
    api.ytdlpVersion().then(setVersion, () => setVersion(null));
    api.shortcutStatus().then(setShortcutError, () => {});
    getVersion().then(setAppVersion, () => {});
  }, []);

  const copyPix = async () => {
    await writeText(support.pixKey);
    setPixCopied(true);
    setTimeout(() => setPixCopied(false), 2000);
  };

  const changeShortcut = useCallback(
    async (shortcut: string) => {
      const ok = await update({ shortcut, shortcutEnabled: true });
      setShortcutError(ok ? null : useSettings.getState().error);
    },
    [update],
  );

  if (!settings) return null;

  const updateYtdlp = async () => {
    setUpdating(true);
    setUpdateMsg(null);
    try {
      const v = await api.updateYtdlp();
      setVersion(v);
      setUpdateMsg({ text: `Atualizado: versão ${v}`, error: false });
    } catch (e) {
      setUpdateMsg({ text: String(e), error: true });
    } finally {
      setUpdating(false);
    }
  };

  return (
    <div className="settings">
      <section>
        <h2>Geral</h2>
        <div className="group">
          <Toggle
            label="Abrir ao iniciar sessão"
            checked={settings.launchAtLogin}
            onChange={(v) => update({ launchAtLogin: v })}
          />
          <Toggle
            label="Atalho global"
            hint="Baixa o link copiado de qualquer app"
            checked={settings.shortcutEnabled}
            onChange={async (v) => {
              const ok = await update({ shortcutEnabled: v });
              setShortcutError(ok ? null : useSettings.getState().error);
            }}
          />
          {settings.shortcutEnabled && (
            <div className={`group-row${shortcutError ? " error" : ""}`}>
              <span className="label">
                <span>Teclas</span>
                <small>{shortcutError ?? "Clique para trocar"}</small>
              </span>
              <ShortcutField value={settings.shortcut} onChange={changeShortcut} />
            </div>
          )}
          <Toggle
            label="Detectar link no clipboard"
            hint="Preenche o campo ao abrir"
            checked={settings.readClipboard}
            onChange={(v) => update({ readClipboard: v })}
          />
          <Toggle
            label="Notificar ao concluir"
            checked={settings.notify}
            onChange={(v) => update({ notify: v })}
          />
        </div>
      </section>

      <section>
        <h2>Downloads</h2>
        <div className="group">
          <div className="group-row">
            <span className="label">
              <span>Pasta de destino</span>
              <small title={settings.outDir}>{shortPath(settings.outDir, home)}</small>
            </span>
            <button type="button" className="secondary-button" onClick={onPickFolder}>
              Alterar…
            </button>
          </div>
          <div className="group-row">
            <span className="label">
              <span>Formato padrão</span>
              <small>{profiles.find((p) => p.id === settings.profileId)?.hint ?? ""}</small>
            </span>
            <label className="popup">
              <select
                aria-label="Formato padrão"
                value={settings.profileId}
                onChange={(e) => update({ profileId: e.target.value })}
              >
                {(["video", "audio"] as const).map((kind) => (
                  <optgroup key={kind} label={kind === "video" ? "Vídeo" : "Áudio"}>
                    {profiles
                      .filter((p) => p.kind === kind)
                      .map((p) => (
                        <option key={p.id} value={p.id}>
                          {p.name}
                        </option>
                      ))}
                  </optgroup>
                ))}
              </select>
              <Icon name="chevron-updown" />
            </label>
          </div>
          <div className="group-row">
            <span className="label">
              <span>Downloads simultâneos</span>
              <small>Quantos vídeos baixam ao mesmo tempo</small>
            </span>
            <label className="popup narrow">
              <select
                aria-label="Downloads simultâneos"
                value={settings.maxParallel}
                onChange={(e) => update({ maxParallel: Number(e.target.value) })}
              >
                {PARALLEL.map((n) => (
                  <option key={n} value={n}>
                    {n}
                  </option>
                ))}
              </select>
              <Icon name="chevron-updown" />
            </label>
          </div>
        </div>
      </section>

      <section>
        <h2>Avançado</h2>
        <div className="group">
          <div className={`group-row${updateMsg?.error ? " error" : ""}`}>
            <span className="label">
              <span>yt-dlp</span>
              <small>
                {updateMsg?.text ??
                  (version === undefined
                    ? "Verificando…"
                    : version
                      ? `Versão ${version}`
                      : "Será baixado no primeiro download")}
              </small>
            </span>
            <button
              type="button"
              className="secondary-button"
              onClick={updateYtdlp}
              disabled={updating || activeCount > 0}
              title={activeCount > 0 ? "Aguarde os downloads terminarem" : undefined}
            >
              {updating && <span className="spinner" aria-hidden="true" />}
              {updating ? "Atualizando…" : version ? "Atualizar" : "Baixar"}
            </button>
          </div>
        </div>
      </section>

      <section>
        <h2>Apoie o desenvolvedor</h2>
        <div className="group">
          <div className="group-row support-text">
            O TubeTray é gratuito e continua assim, sem nada travado. Se ele te ajuda, você pode apoiar o
            desenvolvimento, se quiser.
          </div>
          {(support.sponsorsUrl || support.pixKey) && (
            <div className="group-row license-actions">
              {support.sponsorsUrl && (
                <button type="button" className="secondary-button" onClick={() => openUrl(support.sponsorsUrl)}>
                  GitHub Sponsors
                </button>
              )}
              {support.pixKey && (
                <button type="button" className="secondary-button" onClick={copyPix} title={support.pixKey}>
                  {pixCopied ? "Chave Pix copiada" : "Copiar chave Pix"}
                </button>
              )}
            </div>
          )}
        </div>
      </section>

      <section>
        <h2>Sobre</h2>
        <div className="group">
          <div className="group-row">
            <span className="label">
              <span>TubeTray {appVersion && `v${appVersion}`}</span>
              <small>Código aberto · licença MIT</small>
            </span>
            <button type="button" className="secondary-button" onClick={onOpenLicenses}>
              Licenças
            </button>
          </div>
          <div className="group-row">
            <span className="label">
              <span>Código-fonte e novas versões</span>
              <small>github.com/GuiRoger/tube-tray</small>
            </span>
            <button type="button" className="secondary-button" onClick={() => openUrl(links.repo)}>
              Abrir
            </button>
          </div>
        </div>
      </section>

      {error && error !== shortcutError && (
        <div className="field-message" role="alert">
          {error}
        </div>
      )}
    </div>
  );
}
