// Simula o núcleo Rust para ver a UI no navegador: `npm run dev:mock`.
// Não entra no build do app (só é importado com VITE_MOCK=1).
import { emit } from "@tauri-apps/api/event";
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import type { Job, Settings } from "../lib/tauri";

const params = new URLSearchParams(location.search);
const scenario = params.get("s") ?? "list";

const MB = 1024 * 1024;
const base = { profileId: "mp4-h264", createdAt: Date.now(), file: null, size: null, error: null, speed: null, eta: null };

const sample: Job[] = [
  { ...base, id: "1", url: "https://www.youtube.com/watch?v=aaaaaaaaaaa", stage: "downloading", title: "Como montar um setup minimalista em 2026", percent: 42, speed: 4.8 * MB, eta: 18 },
  { ...base, id: "2", url: "https://www.youtube.com/watch?v=bbbbbbbbbbb", stage: "queued", title: null, percent: 0 },
  { ...base, id: "3", url: "https://www.youtube.com/watch?v=ccccccccccc", stage: "done", title: "Lo-fi beats para programar — 3 horas", percent: 100, file: "C:\\Users\\gui\\Downloads\\TubeTray\\lofi.mp4", size: 186 * MB },
  { ...base, id: "4", url: "https://www.youtube.com/watch?v=ddddddddddd", stage: "failed", title: "Vídeo removido pelo autor", percent: 0, error: "Vídeo indisponível ou removido." },
  { ...base, id: "5", url: "https://www.youtube.com/watch?v=eeeeeeeeeee", stage: "canceled", title: "Review completo do novo notebook", percent: 0 },
  { ...base, id: "6", url: "https://www.youtube.com/watch?v=fffffffffff", stage: "done", title: "Receita de pão de fermentação natural", percent: 100, file: "/x.mp4", size: 1.3 * 1024 * MB },
];

let jobs: Job[] = scenario === "empty" ? [] : sample;
let settings: Settings = {
  outDir: "C:\\Users\\gui\\Downloads\\TubeTray",
  launchAtLogin: false,
  profileId: "mp4-h264",
  readClipboard: true,
  notify: true,
  acceptedNotice: scenario !== "notice",
};

mockWindows("main");
mockIPC(
  (cmd, args) => {
    const a = (args ?? {}) as Record<string, unknown>;
    switch (cmd) {
      case "list_jobs":
        return jobs;
      case "get_settings":
        return settings;
      case "set_settings":
        settings = a.settings as Settings;
        return settings;
      case "enqueue": {
        const job: Job = { ...base, id: String(Math.random()), url: String(a.url), stage: "queued", title: null, percent: 0 };
        jobs = [job, ...jobs];
        setTimeout(() => emit("jobs://update", job), 0);
        return job;
      }
      case "remove_job":
        jobs = jobs.filter((j) => j.id !== a.id);
        return true;
      case "ytdlp_version":
        return "2026.08.19";
      case "plugin:path|resolve_directory":
        return "C:\\Users\\gui";
      case "plugin:clipboard-manager|read_text":
        return scenario === "invalid" ? "https://vimeo.com/123" : "https://youtu.be/dQw4w9WgXcQ";
      default:
        return null;
    }
  },
  { shouldMockEvents: true },
);

if (scenario === "invalid") {
  setTimeout(() => {
    const input = document.querySelector<HTMLInputElement>(".url-field input");
    if (!input) return;
    const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!;
    input.focus();
    setter.call(input, "https://vimeo.com/123");
    input.dispatchEvent(new Event("input", { bubbles: true }));
    input.blur();
  }, 300);
} else if (scenario !== "settings") {
  setTimeout(() => emit("window://shown"), 300);
}
if (scenario === "settings") setTimeout(() => emit("ui://open-settings"), 300);
