// Simula o núcleo Rust para ver a UI no navegador: `npm run dev:mock`.
// Não entra no build do app (só é importado com VITE_MOCK=1).
import { emit } from "@tauri-apps/api/event";
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import type { Job, PlaylistInfo, Profile, Settings } from "../lib/tauri";

const params = new URLSearchParams(location.search);
const scenario = params.get("s") ?? "list";
const hasPreview = scenario.startsWith("preview");
const isPlaylist = scenario.startsWith("playlist");

const profiles: Profile[] = [
  { id: "mp4-h264", kind: "video", name: "Compatível · MP4", hint: "H.264 + AAC, abre em qualquer lugar (até 1080p)" },
  { id: "best", kind: "video", name: "Melhor qualidade", hint: "Até 4K/8K · MP4 ou MKV (AV1/VP9)" },
  { id: "audio-m4a", kind: "audio", name: "M4A · alta", hint: "AAC, sem perda extra" },
  { id: "audio-mp3", kind: "audio", name: "MP3", hint: "VBR ~245 kbps, toca em qualquer aparelho" },
];

const thumb = (a: string, b: string) =>
  "data:image/svg+xml;utf8," +
  encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" width="320" height="180"><defs><linearGradient id="g" x2="1" y2="1"><stop stop-color="${a}"/><stop offset="1" stop-color="${b}"/></linearGradient></defs><rect width="320" height="180" fill="url(#g)"/></svg>`,
  );
const colors = [["#2b5876", "#4e4376"], ["#ff512f", "#dd2476"], ["#11998e", "#38ef7d"], ["#f7971e", "#ffd200"], ["#654ea3", "#eaafc8"]];
const lessons = ["Introdução e instalação", "Ownership na prática", "Borrowing e lifetimes", "Structs e enums", "Pattern matching", "Tratamento de erros", "Traits e generics", "Iteradores e closures", "Async com Tokio", "Projeto final: CLI", "Testes e documentação", "Publicando no crates.io"];
const playlist: PlaylistInfo = {
  id: "PLrust",
  title: "Curso de Rust do zero",
  channel: "Canal Dev Brasil",
  truncated: false,
  entries: lessons.map((t, i) => ({
    id: `vid${String(i).padStart(8, "0")}`,
    url: `https://www.youtube.com/watch?v=vid${String(i).padStart(8, "0")}`,
    title: i === 6 ? "[Private video]" : `Aula ${i + 1} — ${t}`,
    duration: 300 + i * 97,
    thumbnail: thumb(colors[i % 5][0], colors[i % 5][1]),
    available: i !== 6,
  })),
};

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

const parallel: Job[] = [
  { ...base, id: "p1", url: "u1", stage: "downloading", title: "Aula 1 — Introdução e instalação", percent: 64, speed: 3.1 * MB, eta: 22, subdir: "Curso de Rust do zero" },
  { ...base, id: "p2", url: "u2", stage: "downloading", title: "Aula 2 — Ownership na prática", percent: 31, speed: 2.7 * MB, eta: 58, subdir: "Curso de Rust do zero" },
  { ...base, id: "p3", url: "u3", stage: "queued", title: "Aula 3 — Borrowing e lifetimes", percent: 0, subdir: "Curso de Rust do zero" },
  { ...base, id: "p4", url: "u4", profileId: "audio-mp3", stage: "merging", title: "Podcast #42 — Carreira em tecnologia", percent: 100 },
  { ...base, id: "p5", url: "u5", profileId: "audio-m4a", stage: "done", title: "Trilha sonora para estudar", percent: 100, file: "/x.m4a", size: 7.4 * MB },
];

let jobs: Job[] =
  scenario === "empty" ? [] : scenario === "parallel" ? parallel : hasPreview ? sample.slice(2, 6) : isPlaylist ? sample.slice(2, 5) : sample;
let settings: Settings = {
  outDir: "C:\\Users\\gui\\Downloads\\TubeTray",
  launchAtLogin: false,
  profileId: "mp4-h264",
  readClipboard: true,
  notify: true,
  acceptedNotice: scenario !== "notice",
  maxParallel: 2,
  shortcutEnabled: true,
  shortcut: "CommandOrControl+Alt+Y",
};
if (scenario === "audio") settings.profileId = "audio-mp3";

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
      case "enqueue_many": {
        const items = a.items as { url: string; title: string | null }[];
        for (const it of items) {
          const job: Job = { ...base, id: String(Math.random()), url: it.url, profileId: String(a.profileId ?? "mp4-h264"), stage: "queued", title: it.title, percent: 0, subdir: String(a.subdir) };
          jobs = [job, ...jobs];
          setTimeout(() => emit("jobs://update", job), 0);
        }
        return items.length;
      }
      case "profiles":
        return profiles;
      case "playlist_info":
        return new Promise((resolve) => setTimeout(() => resolve(playlist), scenario === "playlist-loading" ? 60000 : 500));
      case "cancel_all": {
        const n = jobs.filter((j) => ["queued", "starting", "downloading", "merging"].includes(j.stage)).length;
        jobs = jobs.map((j) => (["queued", "starting", "downloading", "merging"].includes(j.stage) ? { ...j, stage: "canceled" } : j));
        jobs.forEach((j) => emit("jobs://update", j));
        return n;
      }
      case "shortcut_status":
        return null;
      case "pause_shortcut":
        return null;
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
        if (scenario === "playlist" || scenario === "playlist-loading") return "https://www.youtube.com/playlist?list=PLrust";
        if (scenario === "playlist-video") return "https://www.youtube.com/watch?v=dQw4w9WgXcQ&list=PLrust";
        return hasPreview || scenario === "audio" ? "https://youtu.be/dQw4w9WgXcQ" : "";
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
