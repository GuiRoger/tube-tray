// Wrappers tipados dos comandos e eventos expostos pelo núcleo Rust.
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type Stage =
  | "queued"
  | "starting"
  | "downloading"
  | "merging"
  | "done"
  | "failed"
  | "canceled";

export interface Job {
  id: string;
  url: string;
  profileId: string;
  createdAt: number;
  /** subpasta (nome da playlist), quando veio de uma playlist */
  subdir?: string | null;
  stage: Stage;
  title: string | null;
  /** 0–100 somando vídeo e áudio */
  percent: number;
  /** bytes por segundo */
  speed: number | null;
  /** segundos */
  eta: number | null;
  file: string | null;
  /** bytes do arquivo final */
  size: number | null;
  error: string | null;
  /** concluído, mas o arquivo foi excluído do disco */
  missing: boolean;
}

export interface VideoInfo {
  id: string;
  url: string;
  title: string;
  channel: string;
  thumbnail: string;
}

export type MediaKind = "video" | "audio";

export interface Profile {
  id: string;
  kind: MediaKind;
  name: string;
  hint: string;
}

export interface PlaylistEntry {
  id: string;
  url: string;
  title: string;
  /** segundos */
  duration: number | null;
  thumbnail: string;
  /** privado ou removido */
  available: boolean;
}

export interface PlaylistInfo {
  id: string;
  title: string;
  channel: string;
  entries: PlaylistEntry[];
  truncated: boolean;
}

export interface Settings {
  outDir: string;
  launchAtLogin: boolean;
  profileId: string;
  readClipboard: boolean;
  notify: boolean;
  acceptedNotice: boolean;
  /** downloads ao mesmo tempo (1–5) */
  maxParallel: number;
  shortcutEnabled: boolean;
  /** formato do Tauri: "CommandOrControl+Alt+Y" */
  shortcut: string;
}

export const api = {
  enqueue: (url: string, title?: string | null, profileId?: string | null) =>
    invoke<Job>("enqueue", { url, title: title ?? null, profileId: profileId ?? null }),
  enqueueMany: (items: { url: string; title: string | null }[], profileId: string | null, subdir: string | null) =>
    invoke<number>("enqueue_many", { items, profileId, subdir }),
  profiles: () => invoke<Profile[]>("profiles"),
  playlistInfo: (url: string) => invoke<PlaylistInfo>("playlist_info", { url }),
  shortcutStatus: () => invoke<string | null>("shortcut_status"),
  pauseShortcut: (paused: boolean) => invoke<void>("pause_shortcut", { paused }),
  videoInfo: (url: string) => invoke<VideoInfo>("video_info", { url }),
  refreshFiles: () => invoke<void>("refresh_files"),
  cancel: (id: string) => invoke<boolean>("cancel", { id }),
  cancelAll: () => invoke<number>("cancel_all"),
  retry: (id: string) => invoke<boolean>("retry", { id }),
  clearFinished: () => invoke<Job[]>("clear_finished"),
  removeJob: (id: string) => invoke<boolean>("remove_job", { id }),
  openFile: (path: string) => invoke<void>("open_file", { path }),
  quitApp: () => invoke<void>("quit_app"),
  listJobs: () => invoke<Job[]>("list_jobs"),
  getSettings: () => invoke<Settings>("get_settings"),
  setSettings: (settings: Settings) => invoke<Settings>("set_settings", { settings }),
  revealFile: (path: string) => invoke<void>("reveal_file", { path }),
  openOutDir: () => invoke<void>("open_out_dir"),
  ytdlpVersion: () => invoke<string | null>("ytdlp_version"),
  updateYtdlp: () => invoke<string>("update_ytdlp"),
  hideWindow: () => invoke<void>("hide_window"),
  setPinned: (pinned: boolean) => invoke<void>("set_pinned", { pinned }),
};

export const events = {
  onJob: (cb: (job: Job) => void): Promise<UnlistenFn> =>
    listen<Job>("jobs://update", (e) => cb(e.payload)),
  onYtdlpDownload: (cb: (done: number, total: number | null) => void): Promise<UnlistenFn> =>
    listen<[number, number | null]>("tools://ytdlp-download", (e) => cb(e.payload[0], e.payload[1])),
  onWindowShown: (cb: () => void): Promise<UnlistenFn> => listen("window://shown", () => cb()),
  onOpenSettings: (cb: () => void): Promise<UnlistenFn> => listen("ui://open-settings", () => cb()),
};

export const isActive = (s: Stage) =>
  s === "queued" || s === "starting" || s === "downloading" || s === "merging";
