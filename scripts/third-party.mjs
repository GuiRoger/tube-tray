// Gera src/assets/licenses/third-party.json com nome, versão, licença e
// repositório de todas as dependências Rust e npm que vão no app.
// Rode depois de mudar dependências:  npm run licenses

import { execSync } from "node:child_process";
import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");

// --- Rust: tudo que o pacote do app usa (todas as plataformas)
const meta = JSON.parse(
  execSync("cargo metadata --format-version 1 --manifest-path src-tauri/Cargo.toml", {
    cwd: root,
    maxBuffer: 256 * 1024 * 1024,
  }).toString(),
);
const own = new Set(["tube-tray", "tubetray-core"]);
const rust = meta.packages
  .filter((p) => !own.has(p.name))
  .filter((p) => meta.resolve.nodes.some((n) => n.id === p.id))
  .map((p) => ({
    name: p.name,
    version: p.version,
    license: p.license ?? (p.license_file ? "ver arquivo de licença" : "não informada"),
    url: p.repository ?? p.homepage ?? `https://crates.io/crates/${p.name}`,
    kind: "rust",
  }));

// --- npm: só dependências de produção (as que vão para a interface)
const pkg = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const lock = JSON.parse(readFileSync(join(root, "package-lock.json"), "utf8"));
const seen = new Set();
const npm = [];
const visit = (name) => {
  if (seen.has(name)) return;
  seen.add(name);
  const entry = lock.packages[`node_modules/${name}`];
  if (!entry) return;
  const manifestPath = join(root, "node_modules", name, "package.json");
  const manifest = existsSync(manifestPath) ? JSON.parse(readFileSync(manifestPath, "utf8")) : {};
  const repo = typeof manifest.repository === "string" ? manifest.repository : manifest.repository?.url;
  npm.push({
    name,
    version: entry.version,
    license: entry.license ?? manifest.license ?? "não informada",
    url: (repo ?? manifest.homepage ?? `https://www.npmjs.com/package/${name}`)
      .replace(/^git\+/, "")
      .replace(/\.git$/, "")
      .replace(/^git:\/\//, "https://"),
    kind: "npm",
  });
  for (const dep of Object.keys(entry.dependencies ?? {})) visit(dep);
};
Object.keys(pkg.dependencies ?? {}).forEach(visit);

const all = [...npm, ...rust].sort((a, b) => a.name.localeCompare(b.name));
writeFileSync(join(root, "src/assets/licenses/third-party.json"), JSON.stringify(all) + "\n");
console.log(`${all.length} componentes (${npm.length} npm, ${rust.length} Rust)`);
