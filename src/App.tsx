import { useCallback, useEffect, useRef, useState } from "react";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { homeDir } from "@tauri-apps/api/path";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { api, events } from "./lib/tauri";
import { hasMod } from "./lib/platform";
import { useJobs } from "./store/jobs";
import { selectProfile, useSettings } from "./store/settings";
import { Header } from "./components/Header";
import { UrlForm, type UrlFormHandle } from "./components/UrlForm";
import { DownloadList } from "./components/DownloadList";
import { Footer } from "./components/Footer";
import { SettingsView } from "./components/SettingsView";
import { LicensesView } from "./components/LicensesView";
import { Icon } from "./components/Icon";

const WIDTH = 340;

type View = "main" | "settings" | "licenses";

const TITLES: Record<View, string> = { main: "Baixar vídeo", settings: "Ajustes", licenses: "Licenças" };

export default function App() {
  const [view, setView] = useState<View>("main");
  const [home, setHome] = useState<string | null>(null);
  const settings = useSettings((s) => s.settings);
  const audio = useSettings((s) => selectProfile(s)?.kind === "audio");
  const loadSettings = useSettings((s) => s.load);
  const updateSettings = useSettings((s) => s.update);
  const form = useRef<UrlFormHandle>(null);
  const root = useRef<HTMLDivElement>(null);
  const settingsRef = useRef(settings);
  settingsRef.current = settings;

  // carga inicial + eventos do núcleo
  useEffect(() => {
    const { setAll, upsert, setYtdlpInstall } = useJobs.getState();
    loadSettings();
    api.listJobs().then(setAll);
    homeDir().then(setHome, () => setHome(null));

    const unlisten = [
      events.onJob((job) => {
        upsert(job);
        if (job.stage !== "starting") setYtdlpInstall(null);
      }),
      events.onYtdlpDownload((done, total) => setYtdlpInstall(total ? done / total : 0)),
      events.onWindowShown(() => {
        setView("main");
        if (settingsRef.current?.readClipboard !== false) form.current?.prefillFromClipboard();
        else form.current?.focus();
      }),
      events.onOpenSettings(() => setView("settings")),
    ];
    return () => unlisten.forEach((p) => p.then((f) => f()));
  }, [loadSettings]);

  // confere de tempos em tempos se algum vídeo baixado foi apagado da pasta
  useEffect(() => {
    const id = setInterval(() => {
      if (document.hasFocus()) api.refreshFiles().catch(() => {});
    }, 5000);
    return () => clearInterval(id);
  }, []);

  // a janela acompanha a altura do conteúdo (máx. 520, §7.1)
  useEffect(() => {
    const el = root.current;
    if (!el) return;
    const win = getCurrentWindow();
    let last = 0;
    const ro = new ResizeObserver(() => {
      const h = Math.ceil(el.getBoundingClientRect().height);
      if (h > 0 && h !== last) {
        last = h;
        win.setSize(new LogicalSize(WIDTH, h)).catch(() => {});
      }
    });
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  const pickFolder = useCallback(async () => {
    await api.setPinned(true);
    try {
      const dir = await openDialog({
        directory: true,
        defaultPath: settingsRef.current?.outDir,
        title: "Escolha onde salvar os vídeos",
      });
      if (typeof dir === "string") await updateSettings({ outDir: dir });
    } finally {
      await api.setPinned(false);
      getCurrentWindow().setFocus().catch(() => {});
    }
  }, [updateSettings]);

  // atalhos §8
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        if (view === "licenses") setView("settings");
        else if (view === "settings") setView("main");
        else api.hideWindow();
      } else if (hasMod(e) && e.key.toLowerCase() === "o") {
        e.preventDefault();
        api.openOutDir();
      } else if (hasMod(e) && e.key === ",") {
        e.preventDefault();
        setView((v) => (v === "main" ? "settings" : "main"));
      } else if (hasMod(e) && e.key.toLowerCase() === "q") {
        e.preventDefault();
        api.quitApp();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [view]);

  return (
    <div className="popover" ref={root}>
      <div className="top">
        <Header
          title={view === "main" && audio ? "Baixar áudio" : TITLES[view]}
          onOpenFolder={() => api.openOutDir()}
          onSettings={() => setView((v) => (v === "main" ? "settings" : "main"))}
          settingsActive={view !== "main"}
        />
        {view === "main" && <UrlForm ref={form} />}
      </div>

      {view === "main" ? (
        <>
          {settings && !settings.acceptedNotice && (
            <>
              <div className="separator" />
              <div className="notice" role="note">
                <p>Baixe apenas conteúdo que você tem direito de baixar.</p>
                <button
                  type="button"
                  className="secondary-button"
                  onClick={() => updateSettings({ acceptedNotice: true })}
                >
                  <Icon name="check-circle" />
                  Entendi
                </button>
              </div>
            </>
          )}
          <div className="separator" />
          <DownloadList />
        </>
      ) : (
        view === "settings" ? (
          <SettingsView home={home} onPickFolder={pickFolder} onOpenLicenses={() => setView("licenses")} />
        ) : (
          <LicensesView />
        )
      )}

      <div className="separator" />
      <Footer
        outDir={settings?.outDir ?? ""}
        home={home}
        onPickFolder={pickFolder}
        onQuit={() => api.quitApp()}
      />
    </div>
  );
}
