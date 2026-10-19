import { useMemo, useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import gplText from "../assets/licenses/GPL-3.0.txt?raw";
import mitText from "../assets/licenses/MIT.txt?raw";
import apacheText from "../assets/licenses/Apache-2.0.txt?raw";
import unlicenseText from "../assets/licenses/Unlicense.txt?raw";
import thirdParty from "../assets/licenses/third-party.json";
import { links } from "../lib/support";

const APP_LICENSE = mitText.replace("<year> <copyright holders>", "2026 GuiRoger");

interface Component {
  name: string;
  license: string;
  note: string;
  texts: { label: string; text: string }[];
  source: string;
}

const MAIN: Component[] = [
  {
    name: "TubeTray",
    license: "MIT",
    note: "Este app. Gratuito e de código aberto.",
    texts: [{ label: "MIT", text: APP_LICENSE }],
    source: links.repo,
  },
  {
    name: "yt-dlp",
    license: "Unlicense · executável GPLv3+",
    note: "Faz os downloads. Não vem no instalador: é baixado do GitHub do projeto no primeiro uso e atualizado por você em Ajustes.",
    texts: [
      { label: "Unlicense", text: unlicenseText },
      { label: "GPL-3.0", text: gplText },
    ],
    source: "https://github.com/yt-dlp/yt-dlp",
  },
  {
    name: "FFmpeg",
    license: "GPL-3.0",
    note: "Junta vídeo e áudio em MP4. Vai embutido no instalador (build estático 6.0). O código-fonte correspondente está no site do FFmpeg e também é enviado a quem pedir pelo repositório do TubeTray.",
    texts: [{ label: "GPL-3.0", text: gplText }],
    source: "https://ffmpeg.org/download.html",
  },
  {
    name: "Tauri",
    license: "MIT ou Apache-2.0",
    note: "Base do app (janela, bandeja e integração com o sistema).",
    texts: [
      { label: "MIT", text: mitText },
      { label: "Apache-2.0", text: apacheText },
    ],
    source: "https://github.com/tauri-apps/tauri",
  },
  {
    name: "React",
    license: "MIT",
    note: "Interface.",
    texts: [{ label: "MIT", text: mitText }],
    source: "https://github.com/facebook/react",
  },
];

interface Dep {
  name: string;
  version: string;
  license: string;
  url: string;
}

const open = (url: string) => openUrl(url).catch(() => {});

/** Licenças do TubeTray e dos programas que ele usa. */
export function LicensesView() {
  const [reading, setReading] = useState<{ title: string; text: string } | null>(null);
  const [showAll, setShowAll] = useState(false);
  const deps = thirdParty as Dep[];
  const visibleDeps = useMemo(() => (showAll ? deps : []), [showAll, deps]);

  if (reading) {
    return (
      <div className="settings licenses">
        <button type="button" className="link-button back" onClick={() => setReading(null)}>
          ‹ Voltar às licenças
        </button>
        <h2>{reading.title}</h2>
        <pre className="license-text">{reading.text}</pre>
      </div>
    );
  }

  return (
    <div className="settings licenses">
      <p className="lead">
        O TubeTray é gratuito e de código aberto. Ele usa os programas abaixo, cada um com a sua licença.
      </p>

      {MAIN.map((c) => (
        <section key={c.name} className="group license-card">
          <div className="group-row">
            <span className="label">
              <span className="license-name">{c.name}</span>
              <small>{c.license}</small>
            </span>
          </div>
          <div className="group-row license-note">{c.note}</div>
          <div className="group-row license-actions">
            {c.texts.map((t) => (
              <button
                key={t.label}
                type="button"
                className="secondary-button"
                onClick={() => setReading({ title: `${c.name} — ${t.label}`, text: t.text })}
              >
                Ler {t.label}
              </button>
            ))}
            <button type="button" className="link-button" onClick={() => open(c.source)}>
              Código-fonte
            </button>
          </div>
        </section>
      ))}

      <section>
        <button type="button" className="link-button" onClick={() => setShowAll((v) => !v)}>
          {showAll ? "Esconder" : "Ver"} outros componentes ({deps.length})
        </button>
        {showAll && (
          <ul className="dep-list">
            {visibleDeps.map((d) => (
              <li key={`${d.name}@${d.version}`}>
                <button type="button" className="dep" onClick={() => open(d.url)} title={d.url}>
                  <span>
                    {d.name} <span className="dep-version">{d.version}</span>
                  </span>
                  <span className="dep-license">{d.license}</span>
                </button>
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}
