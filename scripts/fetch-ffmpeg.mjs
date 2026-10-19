// Baixa o ffmpeg estático para src-tauri/binaries/ffmpeg-<target-triple>[.exe],
// nome exigido pelo Tauri para sidecars (bundle.externalBin).
//
// Uso:
//   npm run fetch:ffmpeg                     -> plataforma atual (via rustc)
//   npm run fetch:ffmpeg -- aarch64-apple-darwin

import { execSync } from "node:child_process";
import { createWriteStream, existsSync, mkdirSync, chmodSync, renameSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { pipeline } from "node:stream/promises";
import { Readable } from "node:stream";
import { createGunzip } from "node:zlib";

const RELEASE = "https://github.com/eugeneware/ffmpeg-static/releases/download/b6.0";

const ASSETS = {
  "x86_64-pc-windows-msvc": "ffmpeg-win32-x64.gz",
  "aarch64-apple-darwin": "ffmpeg-darwin-arm64.gz",
  "x86_64-apple-darwin": "ffmpeg-darwin-x64.gz",
  "x86_64-unknown-linux-gnu": "ffmpeg-linux-x64.gz",
  "aarch64-unknown-linux-gnu": "ffmpeg-linux-arm64.gz",
};

function hostTriple() {
  const out = execSync("rustc -vV", { encoding: "utf8" });
  const line = out.split("\n").find((l) => l.startsWith("host:"));
  if (!line) throw new Error("Não foi possível descobrir o target triple (rustc -vV).");
  return line.slice(5).trim();
}

const triple = process.argv[2] ?? hostTriple();
const asset = ASSETS[triple];
if (!asset) {
  console.error(`Target sem ffmpeg configurado: ${triple}\nSuportados: ${Object.keys(ASSETS).join(", ")}`);
  process.exit(1);
}

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const outDir = join(root, "src-tauri", "binaries");
const ext = triple.includes("windows") ? ".exe" : "";
const target = join(outDir, `ffmpeg-${triple}${ext}`);

if (existsSync(target) && !process.argv.includes("--force")) {
  console.log(`ffmpeg já existe: ${target}`);
  process.exit(0);
}

mkdirSync(outDir, { recursive: true });
console.log(`Baixando ${asset} para ${triple}...`);
const res = await fetch(`${RELEASE}/${asset}`);
if (!res.ok || !res.body) {
  console.error(`Falha no download: HTTP ${res.status}`);
  process.exit(1);
}
const tmp = `${target}.download`;
await pipeline(Readable.fromWeb(res.body), createGunzip(), createWriteStream(tmp));
if (!ext) chmodSync(tmp, 0o755);
renameSync(tmp, target);
console.log(`ok: ${target}`);
