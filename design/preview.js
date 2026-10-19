// Gera assets/preview.png — folha de conferência dos ícones
const sharp = require('sharp');
const path = require('path');
const A = (...p) => path.join(__dirname, 'assets', ...p);

(async () => {
  const W = 1400, H = 900;
  const comps = [];
  const bg = `<svg xmlns="http://www.w3.org/2000/svg" width="${W}" height="${H}">
    <rect width="${W}" height="${H}" fill="#F2F2F7"/>
    <rect x="760" y="40" width="600" height="380" rx="24" fill="#1C1C1E"/>
    <rect x="40" y="480" width="660" height="110" rx="16" fill="#ffffff" stroke="#ddd"/>
    <rect x="740" y="480" width="620" height="110" rx="16" fill="#2C2C2E"/>
    <rect x="40" y="640" width="1320" height="110" rx="16" fill="#ffffff" stroke="#ddd"/>
    <rect x="40" y="770" width="1320" height="100" rx="16" fill="#202020"/>
    <g font-family="Segoe UI, Arial" font-size="18" fill="#6e6e73">
      <text x="40" y="470">Menu bar (template) — claro @2x</text>
      <text x="740" y="470">Menu bar (template) — escuro @2x (macOS inverte automaticamente)</text>
      <text x="40" y="630">macOS — 16 · 32 · 64 · 128 · 256 px</text>
    </g></svg>`;
  comps.push({ input: await sharp(Buffer.from(bg)).png().toBuffer(), left: 0, top: 0 });
  comps.push({ input: await sharp(A('png', 'app-icon-1024.png')).resize(400).toBuffer(), left: 40, top: 30 });
  comps.push({ input: await sharp(A('windows', 'app-icon-256.png')).resize(300).toBuffer(), left: 460, top: 80 });
  comps.push({ input: await sharp(A('png', 'app-icon-1024.png')).resize(340).toBuffer(), left: 890, top: 60 });

  const states = ['idle', 'progress-030', 'progress-070', 'done', 'paused', 'error'];
  for (let i = 0; i < states.length; i++) {
    const f = A('macos', 'menubar', `tray-${states[i]}Template@2x.png`);
    comps.push({ input: f, left: 70 + i * 100, top: 517 });
    const inv = await sharp(f).negate({ alpha: false }).toBuffer();
    comps.push({ input: inv, left: 770 + i * 95, top: 517 });
  }
  let x = 70;
  for (const s of [16, 32, 64, 128]) {
    comps.push({ input: A('png', `app-icon-${s}.png`), left: x, top: 695 - s / 2 + 0 });
    x += s + 60;
  }
  x = 70;
  for (const s of [16, 20, 24, 32, 40, 48]) {
    comps.push({ input: A('windows', 'tray', `tray-white-${s}.png`), left: x, top: 820 - s / 2 });
    x += s + 50;
  }
  for (const s of [16, 24, 32, 48]) {
    comps.push({ input: A('windows', `app-icon-${s}.png`), left: x + 100, top: 820 - s / 2 });
    x += s + 50;
  }
  await sharp({ create: { width: W, height: H, channels: 4, background: '#fff' } })
    .composite(comps).png().toFile(A('preview.png'));
  console.log('preview ok');
})();
