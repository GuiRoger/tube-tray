// Simula o núcleo Rust para ver a UI no navegador: `npm run dev:mock`.
// Não entra no build do app (só é importado com VITE_MOCK=1).
import { emit } from "@tauri-apps/api/event";
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import type { Job, Settings } from "../lib/tauri";

const params = new URLSearchParams(location.search);
const scenario = params.get("s") ?? "list";
const hasPreview = scenario.startsWith("preview");

const MB = 1024 * 1024;
const base = { profileId: "mp4-h264", createdAt: Date.now(), file: null, size: null, error: null, speed: null, eta: null, missing: false };

const sample: Job[] = [
  { ...base, id: "1", url: "https://www.youtube.com/watch?v=aaaaaaaaaaa", stage: "downloading", title: "Como montar um setup minimalista em 2026", percent: 42, speed: 4.8 * MB, eta: 18 },
  { ...base, id: "2", url: "https://www.youtube.com/watch?v=bbbbbbbbbbb", stage: "queued", title: null, percent: 0 },
  { ...base, id: "3", url: "https://www.youtube.com/watch?v=ccccccccccc", stage: "done", title: "Aula de Rust: ownership na prática", percent: 100, file: "C:\\Users\\gui\\Downloads\\TubeTray\\aula-rust.mp4", size: 186 * MB },
  { ...base, id: "4", url: "https://www.youtube.com/watch?v=ddddddddddd", stage: "failed", title: "Vídeo removido pelo autor", percent: 0, error: "Vídeo indisponível ou removido." },
  { ...base, id: "5", url: "https://www.youtube.com/watch?v=eeeeeeeeeee", stage: "canceled", title: "Review completo do novo notebook", percent: 0 },
  { ...base, id: "6", url: "https://www.youtube.com/watch?v=fffffffffff", stage: "done", title: "Receita de pão de fermentação natural", percent: 100, file: "/x.mp4", size: 1.3 * 1024 * MB, missing: true },
];

let jobs: Job[] = scenario === "empty" ? [] : hasPreview ? sample.slice(2, 6) : sample;
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
      case "video_info":
        // simula a latência da busca
        return new Promise((resolve, reject) =>
          setTimeout(
            () =>
              scenario === "preview-error"
                ? reject("Vídeo privado ou com restrição.")
                : resolve({
                    id: "dQw4w9WgXcQ",
                    url: String(a.url),
                    title: "Como montar um setup minimalista para home office em 2026 — guia completo",
                    channel: "Canal Setup Brasil",
                    thumbnail: "data:image/svg+xml;utf8," + encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" width="320" height="180"><defs><linearGradient id="g" x2="1" y2="1"><stop stop-color="#2b5876"/><stop offset="1" stop-color="#4e4376"/></linearGradient></defs><rect width="320" height="180" fill="url(#g)"/><circle cx="160" cy="90" r="34" fill="rgba(255,255,255,.85)"/><path d="M150 72v36l30-18z" fill="#4e4376"/></svg>'),
                  }),
            scenario === "preview-loading" ? 60000 : 400,
          ),
        );
      case "plugin:app|version":
        return "0.1.0";
      case "refresh_files":
        return null;
      case "ytdlp_version":
        return "2026.08.19";
      case "plugin:path|resolve_directory":
        return "C:\\Users\\gui";
      case "plugin:clipboard-manager|read_text":
        if (scenario === "invalid") return "https://vimeo.com/123";
        return hasPreview ? "https://youtu.be/dQw4w9WgXcQ" : "";
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
} else if (scenario !== "settings" && !scenario.startsWith("licenses")) {
  setTimeout(() => emit("window://shown"), 300);
}
if (scenario === "settings" || scenario.startsWith("licenses")) setTimeout(() => emit("ui://open-settings"), 300);
if (scenario.startsWith("licenses"))
  setTimeout(() => {
    [...document.querySelectorAll("button")].find((b) => b.textContent === "Licenças")?.click();
    if (scenario === "licenses-text")
      setTimeout(() => [...document.querySelectorAll("button")].find((b) => b.textContent === "Ler GPL-3.0")?.click(), 100);
    if (scenario === "licenses-all")
      setTimeout(() => [...document.querySelectorAll("button")].find((b) => b.textContent?.startsWith("Ver outros"))?.click(), 100);
  }, 500);
