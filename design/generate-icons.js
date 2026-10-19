// Gera todos os assets de ícone do app "YT Tray" (tema Vermelho).
// Uso: node generate-icons.js
const fs = require('fs');
const path = require('path');
const sharp = require('sharp');

const OUT = path.join(__dirname, 'assets');
const dirs = {
  src: 'source-svg',
  mac: 'macos/AppIcon.appiconset',
  macIcns: 'macos',
  menubar: 'macos/menubar',
  win: 'windows',
  winTray: 'windows/tray',
  png: 'png',
  ui: 'ui-icons',
};
for (const d of Object.values(dirs)) fs.mkdirSync(path.join(OUT, d), { recursive: true });
const P = (d, f) => path.join(OUT, dirs[d], f);

/* ------------------------------------------------------------------ */
/* Tokens                                                              */
/* ------------------------------------------------------------------ */
const RED = {
  top: '#FF6B5E',
  mid: '#F0303F',
  bottom: '#B8142F',
  glyphShadow: '#6E0016',
};

/* Superelipse (aprox. do "continuous corner" da Apple, n = 5) */
function squircle(cx, cy, size, n = 4.6, steps = 360) {
  const a = size / 2;
  let d = '';
  for (let i = 0; i <= steps; i++) {
    const t = (i / steps) * Math.PI * 2;
    const c = Math.cos(t), s = Math.sin(t);
    const x = cx + a * Math.sign(c) * Math.pow(Math.abs(c), 2 / n);
    const y = cy + a * Math.sign(s) * Math.pow(Math.abs(s), 2 / n);
    d += (i === 0 ? 'M' : 'L') + x.toFixed(2) + ' ' + y.toFixed(2);
  }
  return d + 'Z';
}

/* ------------------------------------------------------------------ */
/* Ícone do app (1024 canvas)                                          */
/*  - macOS: corpo 824x824 centrado (grid oficial Big Sur+), sombra    */
/*  - fullbleed: corpo 960 para Windows/.ico/web                       */
/* ------------------------------------------------------------------ */
function appIconSVG({ small = false, fullbleed = false } = {}) {
  const body = fullbleed ? 960 : 824;
  const k = body / 824;                     // escala do glifo
  const sw = (small ? 92 : 70) * k;         // stroke mais grosso em 16/32px
  const sq = squircle(512, 512, body);
  const g = (v) => 512 + (v - 512) * k;     // escala em torno do centro
  const glyph = `
    <path d="M${g(512)} ${g(300)} V${g(588)}" />
    <path d="M${g(404)} ${g(482)} L${g(512)} ${g(590)} L${g(620)} ${g(482)}" />
    <path d="M${g(318)} ${g(566)} V${g(648)} Q${g(318)} ${g(714)} ${g(384)} ${g(714)} H${g(640)} Q${g(706)} ${g(714)} ${g(706)} ${g(648)} V${g(566)}" />`;
  return `<svg xmlns="http://www.w3.org/2000/svg" width="1024" height="1024" viewBox="0 0 1024 1024">
  <defs>
    <linearGradient id="bg" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="${RED.top}"/>
      <stop offset="0.55" stop-color="${RED.mid}"/>
      <stop offset="1" stop-color="${RED.bottom}"/>
    </linearGradient>
    <linearGradient id="sheen" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#fff" stop-opacity="${small ? 0.10 : 0.22}"/>
      <stop offset="0.5" stop-color="#fff" stop-opacity="0"/>
    </linearGradient>
    <linearGradient id="glyphFill" gradientUnits="userSpaceOnUse" x1="0" y1="${g(290)}" x2="0" y2="${g(724)}">
      <stop offset="0" stop-color="#FFFFFF"/>
      <stop offset="1" stop-color="#FFF1F0"/>
    </linearGradient>
    <clipPath id="clip"><path d="${sq}"/></clipPath>
    <filter id="shadow" x="-20%" y="-20%" width="140%" height="140%">
      <feDropShadow dx="0" dy="${fullbleed ? 0 : 10}" stdDeviation="${fullbleed ? 0 : 12}" flood-color="#000" flood-opacity="${fullbleed ? 0 : 0.28}"/>
    </filter>
    <filter id="gshadow" x="-30%" y="-30%" width="160%" height="160%">
      <feDropShadow dx="0" dy="${10 * k}" stdDeviation="${14 * k}" flood-color="${RED.glyphShadow}" flood-opacity="${small ? 0 : 0.35}"/>
    </filter>
  </defs>
  <g filter="url(#shadow)"><path d="${sq}" fill="url(#bg)"/></g>
  <g clip-path="url(#clip)">
    <rect x="0" y="0" width="1024" height="1024" fill="url(#sheen)"/>
  </g>
  <path d="${sq}" fill="none" stroke="#fff" stroke-opacity="${small ? 0 : 0.18}" stroke-width="3"/>
  <g filter="url(#gshadow)" fill="none" stroke="url(#glyphFill)" stroke-width="${sw}" stroke-linecap="round" stroke-linejoin="round">${glyph}
  </g>
</svg>`;
}

/* ------------------------------------------------------------------ */
/* Glifos da barra de menu (template, 18x18 pt, preto + alpha)          */
/* ------------------------------------------------------------------ */
const STROKE = 1.6;
const trayGlyph = `
  <path d="M9 2.4V10.2"/>
  <path d="M5.9 7.1L9 10.2L12.1 7.1"/>
  <path d="M3 10.4V13.2A2.2 2.2 0 0 0 5.2 15.4H12.8A2.2 2.2 0 0 0 15 13.2V10.4"/>`;

function tpl(inner, color = '#000', extraDefs = '') {
  return `<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 18 18">${extraDefs}
  <g fill="none" stroke="${color}" stroke-width="${STROKE}" stroke-linecap="round" stroke-linejoin="round">${inner}</g>
</svg>`;
}

function menubarIdle(color) { return tpl(trayGlyph, color); }

// Badge (ponto) com recorte para manter legível em template image
function withBadge(color, badgeInner = '') {
  const defs = `<defs><mask id="m"><rect width="18" height="18" fill="#fff"/><circle cx="14.6" cy="3.6" r="4" fill="#000"/></mask></defs>`;
  return `<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 18 18">${defs}
  <g mask="url(#m)" fill="none" stroke="${color}" stroke-width="${STROKE}" stroke-linecap="round" stroke-linejoin="round">${trayGlyph}</g>
  <circle cx="14.6" cy="3.6" r="2.6" fill="${color}"/>${badgeInner}
</svg>`;
}

function menubarProgress(p, color) {
  const r = 7, c = 2 * Math.PI * r;
  const arc = p <= 0 ? '' : `<circle cx="9" cy="9" r="${r}" stroke-dasharray="${c.toFixed(3)}" stroke-dashoffset="${(c * (1 - p)).toFixed(3)}" transform="rotate(-90 9 9)"/>`;
  return `<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 18 18">
  <g fill="none" stroke="${color}" stroke-width="${STROKE}" stroke-linecap="round" stroke-linejoin="round">
    <circle cx="9" cy="9" r="${r}" stroke-opacity="0.3"/>
    ${arc}
    <path d="M9 5.4V11.6M6.6 9.3L9 11.7L11.4 9.3" stroke-width="1.5"/>
  </g>
</svg>`;
}

function menubarPaused(color) {
  return `<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 18 18">
  <g fill="none" stroke="${color}" stroke-width="${STROKE}" stroke-linecap="round">
    <circle cx="9" cy="9" r="7"/>
    <path d="M7.2 6.3V11.7M10.8 6.3V11.7"/>
  </g>
</svg>`;
}

function menubarError(color) {
  const defs = `<defs><mask id="m"><rect width="18" height="18" fill="#fff"/><circle cx="14" cy="4" r="5" fill="#000"/></mask></defs>`;
  return `<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 18 18">${defs}
  <g mask="url(#m)" fill="none" stroke="${color}" stroke-width="${STROKE}" stroke-linecap="round" stroke-linejoin="round">${trayGlyph}</g>
  <g fill="none" stroke="${color}" stroke-width="1.6" stroke-linecap="round"><path d="M14 1.2V4.6"/></g>
  <circle cx="14" cy="7" r="0.95" fill="${color}"/>
</svg>`;
}

/* ------------------------------------------------------------------ */
/* Ícones de UI (24x24, stroke 1.8) — para stacks web (Electron/Tauri) */
/* Equivalentes SF Symbols no README                                   */
/* ------------------------------------------------------------------ */
const UI = {
  'download':        '<path d="M12 4v11M7.5 10.5 12 15l4.5-4.5M5 20h14"/>',
  'tray-download':   '<path d="M12 3.5v10M7.5 9.5 12 14l4.5-4.5"/><path d="M4 14v3.5A2.5 2.5 0 0 0 6.5 20h11a2.5 2.5 0 0 0 2.5-2.5V14"/>',
  'link':            '<path d="M10 14a5 5 0 0 0 7 0l3-3a5 5 0 0 0-7-7l-1 1M14 10a5 5 0 0 0-7 0l-3 3a5 5 0 0 0 7 7l1-1"/>',
  'clipboard':       '<rect x="6" y="4" width="12" height="17" rx="2.5"/><path d="M9 4.5V3.8A.8.8 0 0 1 9.8 3h4.4a.8.8 0 0 1 .8.8v.7M9.5 11h5M9.5 15h5"/>',
  'folder':          '<path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>',
  'gear':            '<circle cx="12" cy="12" r="3"/><path d="M12 2.8v2.4M12 18.8v2.4M2.8 12h2.4M18.8 12h2.4M5.5 5.5l1.7 1.7M16.8 16.8l1.7 1.7M5.5 18.5l1.7-1.7M16.8 7.2l1.7-1.7"/>',
  'check':           '<path d="m5 12.5 4.5 4.5L19 7.5"/>',
  'check-circle':    '<circle cx="12" cy="12" r="9"/><path d="m8 12.3 2.8 2.8L16.2 9.6"/>',
  'exclamation':     '<circle cx="12" cy="12" r="9"/><path d="M12 7.5v5.5M12 16.5h.01"/>',
  'pause':           '<path d="M9 6v12M15 6v12"/>',
  'play':            '<path d="M8 5.5v13l10.5-6.5z"/>',
  'xmark':           '<path d="M6 6l12 12M18 6 6 18"/>',
  'retry':           '<path d="M4 12a8 8 0 0 1 14-5.3L20 9M20 4v5h-5M20 12a8 8 0 0 1-14 5.3L4 15M4 20v-5h5"/>',
  'magnifier':       '<circle cx="11" cy="11" r="6.5"/><path d="m20 20-4.5-4.5"/>',
  'film':            '<rect x="3.5" y="4" width="17" height="16" rx="2.5"/><path d="M8 4v16M16 4v16M3.5 9H8M3.5 15H8M16 9h4.5M16 15h4.5"/>',
  'music-note':      '<path d="M9 18V5.5l10-2V16"/><circle cx="6.5" cy="18" r="2.5"/><circle cx="16.5" cy="16" r="2.5"/>',
  'chevron-updown':  '<path d="M8 9l4-4 4 4M8 15l4 4 4-4"/>',
  'trash':           '<path d="M4.5 7h15M9.5 7V4.5h5V7M6.5 7l1 13h9l1-13"/>',
};
function uiSVG(inner) {
  return `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">${inner}</svg>\n`;
}

/* ------------------------------------------------------------------ */
/* Writers: ICNS e ICO                                                 */
/* ------------------------------------------------------------------ */
function writeICNS(file, entries /* [{type, buf}] */) {
  const parts = entries.map(({ type, buf }) => {
    const h = Buffer.alloc(8);
    h.write(type, 0, 'ascii');
    h.writeUInt32BE(buf.length + 8, 4);
    return Buffer.concat([h, buf]);
  });
  const body = Buffer.concat(parts);
  const head = Buffer.alloc(8);
  head.write('icns', 0, 'ascii');
  head.writeUInt32BE(body.length + 8, 4);
  fs.writeFileSync(file, Buffer.concat([head, body]));
}

function writeICO(file, images /* [{size, buf}] */) {
  const header = Buffer.alloc(6);
  header.writeUInt16LE(0, 0); header.writeUInt16LE(1, 2); header.writeUInt16LE(images.length, 4);
  let offset = 6 + 16 * images.length;
  const dir = [], data = [];
  for (const { size, buf } of images) {
    const e = Buffer.alloc(16);
    e.writeUInt8(size >= 256 ? 0 : size, 0);
    e.writeUInt8(size >= 256 ? 0 : size, 1);
    e.writeUInt8(0, 2); e.writeUInt8(0, 3);
    e.writeUInt16LE(1, 4); e.writeUInt16LE(32, 6);
    e.writeUInt32LE(buf.length, 8); e.writeUInt32LE(offset, 12);
    offset += buf.length; dir.push(e); data.push(buf);
  }
  fs.writeFileSync(file, Buffer.concat([header, ...dir, ...data]));
}

const render = (svg, px) => sharp(Buffer.from(svg), {
    density: Math.min(2400, (72 * px * 2) / Number(/width="(\d+)"/.exec(svg)[1])),
  })
  .resize(px, px, { fit: 'contain', background: { r: 0, g: 0, b: 0, alpha: 0 } })
  .png({ compressionLevel: 9 })
  .toBuffer();

/* ------------------------------------------------------------------ */
(async () => {
  const appSVG = appIconSVG();
  const appSmallSVG = appIconSVG({ small: true });
  const fullSVG = appIconSVG({ fullbleed: true });
  const fullSmallSVG = appIconSVG({ fullbleed: true, small: true });

  // Fontes SVG
  fs.writeFileSync(P('src', 'app-icon-macos.svg'), appSVG);
  fs.writeFileSync(P('src', 'app-icon-macos-small.svg'), appSmallSVG);
  fs.writeFileSync(P('src', 'app-icon-fullbleed.svg'), fullSVG);
  fs.writeFileSync(P('src', 'menubar-idle.svg'), menubarIdle('#000'));
  fs.writeFileSync(P('src', 'menubar-done.svg'), withBadge('#000'));
  fs.writeFileSync(P('src', 'menubar-error.svg'), menubarError('#000'));
  fs.writeFileSync(P('src', 'menubar-paused.svg'), menubarPaused('#000'));
  fs.writeFileSync(P('src', 'menubar-progress-50.svg'), menubarProgress(0.5, '#000'));

  const appPNG = async (px) => render(px <= 32 ? appSmallSVG : appSVG, px);
  const fullPNG = async (px) => render(px <= 32 ? fullSmallSVG : fullSVG, px);

  // PNGs avulsos do ícone macOS
  for (const s of [16, 32, 64, 128, 256, 512, 1024]) {
    fs.writeFileSync(P('png', `app-icon-${s}.png`), await appPNG(s));
  }

  // AppIcon.appiconset (Xcode)
  const set = [[16,1],[16,2],[32,1],[32,2],[128,1],[128,2],[256,1],[256,2],[512,1],[512,2]];
  const images = [];
  for (const [pt, sc] of set) {
    const name = `icon_${pt}x${pt}${sc === 2 ? '@2x' : ''}.png`;
    fs.writeFileSync(P('mac', name), await appPNG(pt * sc));
    images.push({ idiom: 'mac', size: `${pt}x${pt}`, scale: `${sc}x`, filename: name });
  }
  fs.writeFileSync(P('mac', 'Contents.json'), JSON.stringify({ images, info: { version: 1, author: 'xcode' } }, null, 2));

  // .icns
  const icnsMap = [['icp4',16],['icp5',32],['icp6',64],['ic07',128],['ic08',256],['ic09',512],['ic10',1024],['ic11',32],['ic12',64],['ic13',256],['ic14',512]];
  const icnsEntries = [];
  for (const [type, px] of icnsMap) icnsEntries.push({ type, buf: await appPNG(px) });
  writeICNS(P('macIcns', 'AppIcon.icns'), icnsEntries);

  // Menu bar (template) — Electron: nome terminando em "Template" => macOS inverte cor automaticamente
  const mb = {
    idle: menubarIdle('#000'),
    done: withBadge('#000'),
    error: menubarError('#000'),
    paused: menubarPaused('#000'),
  };
  for (let i = 0; i <= 10; i++) mb[`progress-${String(i * 10).padStart(3, '0')}`] = menubarProgress(i / 10, '#000');
  for (const [k, svg] of Object.entries(mb)) {
    const base = `tray-${k}Template`;
    fs.writeFileSync(P('menubar', `${base}.png`), await render(svg, 18));
    fs.writeFileSync(P('menubar', `${base}@2x.png`), await render(svg, 36));
    fs.writeFileSync(P('menubar', `${base}@3x.png`), await render(svg, 54));
  }

  // Windows: app .ico (full-bleed) + PNGs
  const icoSizes = [16, 20, 24, 32, 40, 48, 64, 128, 256];
  const icoImgs = [];
  for (const s of icoSizes) {
    const buf = await fullPNG(s);
    icoImgs.push({ size: s, buf });
    fs.writeFileSync(P('win', `app-icon-${s}.png`), buf);
  }
  writeICO(P('win', 'app.ico'), icoImgs);

  // Windows tray: claro (glifo branco p/ taskbar escura) e escuro (glifo preto p/ taskbar clara) + versão colorida
  const traySizes = [16, 20, 24, 32, 40, 48];
  for (const [variant, color] of [['white', '#FFFFFF'], ['black', '#000000'], ['red', '#FF3B30']]) {
    for (const state of ['idle', 'done', 'error', 'paused']) {
      const svgFn = { idle: menubarIdle, done: withBadge, error: menubarError, paused: menubarPaused }[state];
      const svg = svgFn(color);
      const imgs = [];
      for (const s of traySizes) {
        const buf = await render(svg, s);
        imgs.push({ size: s, buf });
        if (state === 'idle') fs.writeFileSync(P('winTray', `tray-${variant}-${s}.png`), buf);
      }
      writeICO(P('winTray', `tray-${state}-${variant}.ico`), imgs);
    }
  }

  // Ícones de UI
  for (const [name, inner] of Object.entries(UI)) fs.writeFileSync(P('ui', `${name}.svg`), uiSVG(inner));

  console.log('OK — assets gerados em', OUT);
})().catch((e) => { console.error(e); process.exit(1); });
