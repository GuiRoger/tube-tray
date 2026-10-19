# Design System — App de Menu Bar para baixar vídeos do YouTube

> **Como usar este documento:** cole-o inteiro em outro chat como contexto. Ele descreve o produto, os tokens visuais, os componentes, os comportamentos e os assets de ícone já gerados.
> Nome do app ainda não definido. Neste documento ele aparece como **`{AppName}`**.

---

## 1. Produto

| Item | Definição |
|---|---|
| Tipo | App de **barra de menu** (tray / menu bar extra), sem ícone no Dock por padrão (`LSUIElement = YES`) |
| Função | O usuário cola um link do YouTube → escolhe Vídeo/Áudio + qualidade → baixa |
| Plataforma principal | macOS 13+ (visual nativo seguindo as Apple HIG). Windows é opcional, já tem assets |
| Linguagem visual | macOS nativo: materiais com vibrancy, SF Pro, SF Symbols, controles padrão do AppKit/SwiftUI |
| Cor de marca | **Vermelho do sistema Apple** (`systemRed`) |
| Stack sugerida | **SwiftUI `MenuBarExtra` (`.menuBarExtraStyle(.window)`)** + `yt-dlp` + `ffmpeg` embutidos. Alternativa: Electron/Tauri (tokens CSS abaixo) |

### Princípios
1. **Um gesto:** abrir → colar → baixar. Nada além disso na tela principal.
2. **Nativo antes de tudo:** usar os controles do sistema; nada de UI "web" customizada.
3. **Discreto:** o ícone da barra é monocromático (template). A cor vermelha aparece só dentro do popover e no ícone do app.
4. **Feedback contínuo:** o progresso aparece no ícone da barra, na lista e na notificação ao concluir.

---

## 2. Cores

### 2.1 Destaque (marca)
| Token | Light | Dark | Uso |
|---|---|---|---|
| `accent` | `#FF3B30` | `#FF453A` | Botão primário, progresso, foco, seleção |
| `accentHover` | `#F2362B` | `#FF5A50` | Hover do botão primário |
| `accentPressed` | `#D92F26` | `#E63E34` | Botão pressionado |
| `accentSubtle` | `rgba(255,59,48,0.12)` | `rgba(255,69,58,0.18)` | Fundo de ícone em linha "baixando", chips |
| `accentHighContrast` | `#D70015` | `#FF6961` | Usar quando *Aumentar contraste* estiver ativo |
| `onAccent` | `#FFFFFF` | `#FFFFFF` | Texto e ícone sobre accent |
| `focusRing` | `accent` @ 40% | `accent` @ 40% | Anel de foco de 3px nos campos |

### 2.2 Ícone do app (gradiente)
| Stop | Cor |
|---|---|
| 0% (topo) | `#FF6B5E` |
| 55% | `#F0303F` |
| 100% (base) | `#B8142F` |
| Glifo | Branco `#FFFFFF` → `#FFF1F0`, sombra `#6E0016` @35% |

### 2.3 Status
| Token | Light | Dark | Observação |
|---|---|---|---|
| `success` | `#34C759` | `#30D158` | Concluído |
| `warning` | `#FF9500` | `#FF9F0A` | Pausado, aviso |
| `error` | `#D70015` | `#FF6961` | Falha. **Sempre com ícone + texto** (ver nota) |
| `info` | `#007AFF` | `#0A84FF` | Links, dicas |

> ⚠️ **Accent e erro são da mesma família.** Por isso um erro **nunca** é indicado só pela cor: use sempre `exclamationmark.triangle.fill` junto com uma mensagem. Ações destrutivas (remover da lista) usam texto vermelho apenas em diálogos de confirmação.

### 2.4 Neutros semânticos (equivalentes ao NSColor)
| Token | Light | Dark | NSColor |
|---|---|---|---|
| `label` | `rgba(0,0,0,0.85)` | `rgba(255,255,255,0.85)` | `labelColor` |
| `secondaryLabel` | `rgba(60,60,67,0.60)` | `rgba(235,235,245,0.60)` | `secondaryLabelColor` |
| `tertiaryLabel` | `rgba(60,60,67,0.30)` | `rgba(235,235,245,0.30)` | `tertiaryLabelColor` |
| `separator` | `rgba(0,0,0,0.10)` | `rgba(255,255,255,0.10)` | `separatorColor` |
| `popoverBackground` | `rgba(242,242,247,0.80)` + blur | `rgba(36,36,38,0.74)` + blur | `.popover` material |
| `fill` (controles) | `rgba(0,0,0,0.05)` | `rgba(255,255,255,0.08)` | `quaternarySystemFill` |
| `fieldBackground` | `#FFFFFF` | `rgba(255,255,255,0.06)` | `textBackgroundColor` |
| `fieldBorder` | `rgba(0,0,0,0.12)` | `rgba(255,255,255,0.10)` | — |
| `segmentSelected` | `#FFFFFF` | `rgba(255,255,255,0.20)` | — |
| `progressTrack` | `rgba(0,0,0,0.08)` | `rgba(255,255,255,0.12)` | — |
| `rowHover` | `rgba(0,0,0,0.04)` | `rgba(255,255,255,0.05)` | — |

---

## 3. Tipografia
Fonte: **SF Pro** (do sistema). Na web: `-apple-system, BlinkMacSystemFont, "SF Pro Text", "Segoe UI Variable", system-ui, sans-serif`.

| Estilo | Tamanho / peso | Line height | Uso |
|---|---|---|---|
| Large Title | 26pt Regular | 32 | Título da janela de Ajustes (raro) |
| Title 3 | 15pt Semibold | 20 | Cabeçalho de seção em Ajustes |
| Headline | 13pt Semibold | 16 | Título do popover, botão primário |
| Body | 13pt Regular | 16 | Campos, títulos de vídeo, linhas da lista |
| Callout / Subheadline | 12pt Regular | 15 | Metadados (canal, duração), status |
| Caption | 11pt Regular | 13 | Rodapé, atalhos, contadores |
| Monospaced digits | 12pt `.monospacedDigit()` | — | %, MB/s, tempo restante (evita "tremer") |

---

## 4. Espaçamento, raios e elevação

**Grid de 4pt:** `2 · 4 · 8 · 12 · 16 · 20 · 24`

| Token | Valor | Uso |
|---|---|---|
| `space.xs` | 4 | Ícone ↔ texto pequeno |
| `space.sm` | 8 | Entre controles relacionados |
| `space.md` | 12 | **Padding interno do popover**, entre blocos |
| `space.lg` | 16 | Margens da janela de Ajustes |
| `space.xl` | 20 | Seções em Ajustes |

| Raio | Valor | Uso |
|---|---|---|
| `radius.xs` | 4 | Badges e keycaps (⌘V) |
| `radius.sm` | 6 | Botões, popup button, linhas da lista (hover) |
| `radius.md` | 7 | Campo de URL, botão primário |
| `radius.lg` | 10 | Card de preview do vídeo, janela de Ajustes |
| `radius.xl` | 12 | Popover |
| App icon | Squircle (superelipse n≈4,6) | Grid Apple de 824/1024 |

**Elevação (popover):** `0 20px 60px rgba(0,0,0,.25), 0 0 0 .5px rgba(0,0,0,.15)` no light; `0 20px 60px rgba(0,0,0,.6), 0 0 0 .5px rgba(0,0,0,.8)` + borda interna `rgba(255,255,255,.12)` no dark.
**Material:** `backdrop-filter: blur(30px) saturate(180%)`. No nativo use `NSVisualEffectView.material = .popover` (ou o padrão do `MenuBarExtra`).

---

## 5. Motion
| Token | Valor | Uso |
|---|---|---|
| `duration.fast` | 120ms | Hover, pressed |
| `duration.base` | 200ms | Troca de segmentos, aparecer o card de preview |
| `duration.slow` | 300ms | Inserir ou remover linha da lista |
| `easing.standard` | `cubic-bezier(0.2, 0.8, 0.2, 1)` | Padrão |
| Nativo | `.spring(response: 0.3, dampingFraction: 0.85)` | SwiftUI |

O anel de progresso no ícone da barra é atualizado no máximo **a cada 250ms**. Respeitar **Reduzir movimento**: sem springs, apenas fade.

---

## 6. Ícones

### 6.1 Ícone do app
Squircle vermelho com gradiente (§2.2) e glifo branco de **seta para baixo entrando numa bandeja** (`tray.and.arrow.down`). Os tamanhos ≤32px usam traço mais grosso e não têm brilho nem sombra no glifo.

### 6.2 Ícone da barra de menu (template)
- Canvas **18×18pt**, traço **1,6pt**, cantos e pontas arredondados, **preto + alpha**.
- Marcado como **template image** (o macOS inverte automaticamente no modo escuro e quando o item é destacado). **Nunca colorir.**
- Estados:

| Estado | Arquivo | Visual |
|---|---|---|
| Ocioso | `tray-idleTemplate` | Seta + bandeja |
| Baixando | `tray-progress-000…100Template` (11 frames, de 10 em 10%) | Anel de progresso + seta pequena |
| Concluído (não visto) | `tray-doneTemplate` | Ícone ocioso + ponto no canto superior direito |
| Pausado | `tray-pausedTemplate` | Círculo com ‖ |
| Erro | `tray-errorTemplate` | Ícone ocioso + "!" no canto |

O ponto de "concluído" some quando o usuário abre o popover. Com vários downloads ao mesmo tempo, o anel mostra a **média ponderada por bytes**.

### 6.3 Ícones de UI (SF Symbols → SVG equivalentes em `ui-icons/`)
| Uso | SF Symbol | SVG |
|---|---|---|
| Baixar | `arrow.down.to.line` | `download.svg` |
| Marca | `tray.and.arrow.down` | `tray-download.svg` |
| Link | `link` | `link.svg` |
| Colar | `doc.on.clipboard` | `clipboard.svg` |
| Abrir pasta | `folder` | `folder.svg` |
| Ajustes | `gearshape` | `gear.svg` |
| Concluído | `checkmark` / `checkmark.circle.fill` | `check.svg` / `check-circle.svg` |
| Erro | `exclamationmark.triangle.fill` | `exclamation.svg` |
| Pausar / Retomar | `pause.fill` / `play.fill` | `pause.svg` / `play.svg` |
| Cancelar | `xmark` | `xmark.svg` |
| Tentar de novo | `arrow.clockwise` | `retry.svg` |
| Mostrar no Finder | `magnifyingglass` | `magnifier.svg` |
| Vídeo / Áudio | `film` / `music.note` | `film.svg` / `music-note.svg` |
| Popup | `chevron.up.chevron.down` | `chevron-updown.svg` |
| Remover | `trash` | `trash.svg` |

SVGs: viewBox 24, `stroke="currentColor"`, traço 1,8, cantos e pontas arredondados.

---

## 7. Componentes

### 7.1 Popover (tela principal)
- **Largura 340pt**, altura dinâmica (máx. 520pt; a lista rola dentro dela), padding 12, raio 12.
- Ancorado no ícone da barra. Fecha ao clicar fora ou com `Esc`.

```
┌──────────────────────────────────────┐
│ [■] Baixar vídeo            📁  ⚙︎    │  Header
│ ┌──────────────────────────────────┐ │
│ │ 🔗 youtube.com/watch?v=…     ⌘V  │ │  URL field
│ └──────────────────────────────────┘ │
│ ┌──────┐ Título do vídeo…            │  Preview card
│ │ ▶︎    │ Canal · 1080p disponível    │
│ └──────┘                             │
│ [ Vídeo | Áudio ]         [1080p·MP4⌄]│  Opções
│ [          ⬇︎  Baixar               ] │  Botão primário
├──────────────────────────────────────┤
│ Recentes                             │
│ (⬇︎) Nome…  ▬▬▬▬▬▬░░░  42% · 3 MB/s   │  Download row
│ (✓) Nome…  Concluído · 186 MB    🔍  │
│ (!) Nome…  Falhou · Indisponível  ↻  │
├──────────────────────────────────────┤
│ Salvar em: ~/Downloads       Sair ⌘Q │  Footer
└──────────────────────────────────────┘
```

### 7.2 Header
- Mini ícone do app 24×24 (raio 6) + "Baixar vídeo" (Headline).
- À direita, dois botões ícone 28×28 (raio 6, hover `rowHover`) em `secondaryLabel`: **Abrir pasta** (⌘O) e **Ajustes** (⌘,).

### 7.3 Campo de URL
- Altura 32, raio 7, fundo `fieldBackground`, borda 0,5px `fieldBorder`, foco com anel `focusRing` de 3px.
- Ícone `link` 14pt à esquerda em `secondaryLabel`. Placeholder: *"Cole o link do YouTube…"*. À direita, keycap "⌘V" (Caption, fundo `fill`, raio 4).
- **Detecção automática:** ao abrir o popover, se o clipboard contém uma URL do YouTube, o campo vem preenchido com o texto selecionado.
- Validação: aceitar `youtube.com/watch`, `youtu.be/`, `/shorts/`, `/live/` e playlists (`list=`). Se o link for inválido, aparece abaixo do campo, em Callout com a cor `error` e ícone: *"Esse link não parece ser do YouTube."*

### 7.4 Card de preview
- Aparece com fade e slide de 4pt (200ms) depois que os metadados chegam. Enquanto carrega, mostra skeleton com shimmer.
- Fundo `fill`, raio 10, padding 8. Thumbnail 84×48 (raio 6) com a duração no canto (Caption, branco, sobre preto 60%).
- Título em Body Medium, 1 linha, truncado. Abaixo, "Canal · melhor qualidade" em Callout `secondaryLabel`.
- Para playlist: *"Playlist · 24 vídeos"*, e o botão vira *"Baixar 24 vídeos"*.

### 7.5 Opções
- **Segmented control** com Vídeo | Áudio: altura 24, fundo `fill`, raio 7; o segmento selecionado fica em `segmentSelected` com sombra `0 .5px 1.5px rgba(0,0,0,.2)`.
- **Popup button** de qualidade (altura 28, raio 6, com chevron up/down):
  - Vídeo: `Melhor`, `2160p`, `1440p`, `1080p` (padrão), `720p`, `480p` × `MP4` (padrão) / `WebM` / `MKV`
  - Áudio: `Alta · M4A` (padrão), `MP3 320`, `MP3 192`, `Opus`, `WAV`
- Opções indisponíveis para o vídeo aparecem desabilitadas.

### 7.6 Botão primário
- Largura total, altura 32, raio 7, fundo `accent`, texto `onAccent` em Headline, ícone de download 14pt.
- Estados: hover `accentHover`; pressed `accentPressed` + `scale(.99)`; desabilitado com opacidade 40% (sem URL válida); durante a preparação, texto *"Preparando…"* com spinner.
- Atalho: `Return`.

### 7.7 Linha de download
- Altura ~52, padding 6, raio 6, hover `rowHover`.
- Ícone 28×28 (raio 6) à esquerda:
  - Baixando: fundo `accentSubtle`, seta em `accent`
  - Concluído: fundo `success` @16%, check em `success`
  - Erro: fundo `error` @15%, "!" em `error`
  - Pausado: fundo `warning` @15%, ‖ em `warning`
- Texto: título em Body (1 linha). Barra de progresso de **4px** (raio total, trilho `progressTrack`, preenchimento `accent`). Status em Callout com dígitos monoespaçados: `42% · 4,8 MB/s · 18s restantes`.
- Ações aparecem no hover (ícones de 14pt em `tertiaryLabel` → `label`):
  - Baixando: Pausar, Cancelar
  - Concluído: Mostrar no Finder, Abrir
  - Erro: Tentar de novo
- Duplo clique abre o arquivo. Menu de contexto: Abrir · Mostrar no Finder · Copiar link · Remover da lista.
- Mostrar no máximo 5 recentes, com o link *"Ver todos"*.

### 7.8 Estado vazio (sem downloads)
Ícone `tray.and.arrow.down` de 28pt em `tertiaryLabel`, *"Nenhum download ainda"* em Body `secondaryLabel` e *"Copie um link do YouTube e ele aparece aqui."* em Callout.

### 7.9 Footer
Altura 32, padding 12 × 8, Callout `secondaryLabel`. À esquerda, *"Salvar em: ~/Downloads"* (clicável, abre o seletor de pasta). À direita, *"Sair ⌘Q"*.

### 7.10 Menu de contexto (clique direito no ícone da barra)
`Abrir {AppName}` · `Colar e baixar (⇧⌘V)` · — · `Abrir pasta de downloads` · `Ajustes… (⌘,)` · — · `Sair (⌘Q)`

### 7.11 Janela de Ajustes
Usar `Settings { }` do SwiftUI com `Form` e `.formStyle(.grouped)`. Largura 480. Abas na toolbar:
- **Geral:** Abrir ao iniciar sessão · Atalho global (padrão `⌥⌘D`) · Mostrar ícone no Dock · Detectar link no clipboard
- **Downloads:** Pasta de destino · Formato e qualidade padrão · Nome do arquivo (`%(title)s.%(ext)s`) · Downloads simultâneos (1–5, padrão 2) · Baixar legendas
- **Avançado:** Atualizar o yt-dlp automaticamente · Limite de banda · Cookies do navegador (para vídeos com restrição de idade) · Logs

### 7.12 Notificações
Ao concluir, enviar uma `UNUserNotification` com título *"Download concluído"*, corpo com o título do vídeo e a ação **Mostrar no Finder**. Em caso de erro: título *"Falha no download"* e ação **Tentar de novo**.

---

## 8. Interações e atalhos
| Ação | Atalho |
|---|---|
| Abrir/fechar o popover de qualquer lugar | `⌥⌘D` (global, configurável) |
| Colar link | `⌘V` |
| Baixar | `Return` |
| Alternar Vídeo/Áudio | `⌘1` / `⌘2` |
| Abrir pasta | `⌘O` |
| Ajustes | `⌘,` |
| Fechar popover | `Esc` |
| Sair | `⌘Q` |
| **Arrastar link** para o ícone da barra | Inicia o download com o formato padrão |

---

## 9. Acessibilidade
- Branco sobre `#FF3B30` tem contraste de **3,55:1**. Por isso o texto do botão é **Semibold**. Com *Aumentar contraste* ativo, trocar para `accentHighContrast` (`#D70015`, 5,4:1).
- Todo botão que tem só ícone precisa de `accessibilityLabel` ("Abrir pasta", "Ajustes", "Pausar download").
- O progresso é anunciado no VoiceOver a cada 25%.
- Suportar teclado completo (Tab entre os controles), Dynamic Type do macOS e *Reduzir transparência* (trocar o material por um fundo sólido `#F2F2F7` / `#242426`).
- O estado nunca é transmitido só pela cor: sempre ícone + texto.

---

## 10. Tokens prontos para código

### CSS (Electron/Tauri)
```css
.theme-light {
  --accent:#FF3B30; --accent-hover:#F2362B; --accent-pressed:#D92F26; --accent-subtle:rgba(255,59,48,.12); --on-accent:#fff;
  --success:#34C759; --warning:#FF9500; --error:#D70015; --info:#007AFF;
  --label:rgba(0,0,0,.85); --label-2:rgba(60,60,67,.6); --label-3:rgba(60,60,67,.3);
  --separator:rgba(0,0,0,.1); --fill:rgba(0,0,0,.05); --field:#fff; --field-border:rgba(0,0,0,.12);
  --segment-selected:#fff; --track:rgba(0,0,0,.08); --row-hover:rgba(0,0,0,.04);
  --popover:rgba(242,242,247,.8);
}
.theme-dark {
  --accent:#FF453A; --accent-hover:#FF5A50; --accent-pressed:#E63E34; --accent-subtle:rgba(255,69,58,.18); --on-accent:#fff;
  --success:#30D158; --warning:#FF9F0A; --error:#FF6961; --info:#0A84FF;
  --label:rgba(255,255,255,.85); --label-2:rgba(235,235,245,.6); --label-3:rgba(235,235,245,.3);
  --separator:rgba(255,255,255,.1); --fill:rgba(255,255,255,.08); --field:rgba(255,255,255,.06); --field-border:rgba(255,255,255,.1);
  --segment-selected:rgba(255,255,255,.2); --track:rgba(255,255,255,.12); --row-hover:rgba(255,255,255,.05);
  --popover:rgba(36,36,38,.74);
}
:root {
  --font: -apple-system, BlinkMacSystemFont, "SF Pro Text", "Segoe UI Variable", system-ui, sans-serif;
  --fs-large:26px; --fs-title3:15px; --fs-body:13px; --fs-callout:12px; --fs-caption:11px;
  --sp-1:4px; --sp-2:8px; --sp-3:12px; --sp-4:16px; --sp-5:20px; --sp-6:24px;
  --r-xs:4px; --r-sm:6px; --r-md:7px; --r-lg:10px; --r-xl:12px;
  --dur-fast:120ms; --dur-base:200ms; --dur-slow:300ms; --ease:cubic-bezier(.2,.8,.2,1);
  --material: blur(30px) saturate(180%);
}
```

### SwiftUI
```swift
import SwiftUI

enum DS {
    // Cores: o accent vem do Asset Catalog ("AccentColor" = #FF3B30 / #FF453A)
    static let accent = Color.accentColor            // ou Color(nsColor: .systemRed)
    static let success = Color(nsColor: .systemGreen)
    static let warning = Color(nsColor: .systemOrange)
    static let error = Color(nsColor: .systemRed)
    static let label = Color(nsColor: .labelColor)
    static let secondaryLabel = Color(nsColor: .secondaryLabelColor)
    static let tertiaryLabel = Color(nsColor: .tertiaryLabelColor)
    static let fill = Color(nsColor: .quaternarySystemFill)
    static let separator = Color(nsColor: .separatorColor)

    enum Space { static let xs: CGFloat = 4, sm: CGFloat = 8, md: CGFloat = 12, lg: CGFloat = 16, xl: CGFloat = 20 }
    enum Radius { static let xs: CGFloat = 4, sm: CGFloat = 6, md: CGFloat = 7, lg: CGFloat = 10, xl: CGFloat = 12 }
    enum Size {
        static let popoverWidth: CGFloat = 340
        static let popoverMaxHeight: CGFloat = 520
        static let field: CGFloat = 32
        static let primaryButton: CGFloat = 32
        static let rowIcon: CGFloat = 28
        static let progressHeight: CGFloat = 4
        static let thumb = CGSize(width: 84, height: 48)
    }
    enum Motion { static let spring = Animation.spring(response: 0.3, dampingFraction: 0.85) }
}

// Menu bar
// MenuBarExtra { ContentView() } label: { Image("tray-idleTemplate") }
//     .menuBarExtraStyle(.window)
// Info.plist → LSUIElement = YES
```

### JSON (design tokens)
```json
{
  "color": {
    "accent":   { "light": "#FF3B30", "dark": "#FF453A" },
    "accentHC": { "light": "#D70015", "dark": "#FF6961" },
    "success":  { "light": "#34C759", "dark": "#30D158" },
    "warning":  { "light": "#FF9500", "dark": "#FF9F0A" },
    "error":    { "light": "#D70015", "dark": "#FF6961" },
    "iconGradient": ["#FF6B5E", "#F0303F", "#B8142F"]
  },
  "font": { "family": "SF Pro", "body": 13, "callout": 12, "caption": 11, "headline": 13, "title3": 15, "largeTitle": 26 },
  "space": [2, 4, 8, 12, 16, 20, 24],
  "radius": { "xs": 4, "sm": 6, "md": 7, "lg": 10, "xl": 12 },
  "size": { "popoverWidth": 340, "popoverMaxHeight": 520, "field": 32, "button": 32, "menubarGlyph": 18, "menubarStroke": 1.6 },
  "motion": { "fast": 120, "base": 200, "slow": 300, "easing": "cubic-bezier(0.2,0.8,0.2,1)" }
}
```

---

## 11. Assets gerados
Pasta: `C:\Users\guilh\.gemini\antigravity\scratch\yt-tray-design\assets\`. Para regerar: `node generate-icons.js`.

```
assets/
├── source-svg/                 Fontes vetoriais (editáveis)
│   ├── app-icon-macos.svg          1024, grid Apple (corpo 824 + sombra)
│   ├── app-icon-macos-small.svg    versão para 16/32px (traço grosso)
│   ├── app-icon-fullbleed.svg      sem margem (Windows / web)
│   └── menubar-*.svg               idle, done, error, paused, progress-50
├── macos/
│   ├── AppIcon.icns                pronto para Electron/Tauri/Xcode
│   ├── AppIcon.appiconset/         arrastar para Assets.xcassets (Contents.json incluso)
│   └── menubar/                    tray-{idle,done,error,paused,progress-000…100}Template{,@2x,@3x}.png
├── windows/
│   ├── app.ico                     16–256 px
│   ├── app-icon-{16…256}.png
│   └── tray/                       tray-{state}-{white|black|red}.ico + PNGs 16–48
├── png/app-icon-{16…1024}.png
└── ui-icons/*.svg                  18 ícones de interface (currentColor)
```

**Electron:** `new Tray(path.join(__dirname,'assets/macos/menubar/tray-idleTemplate.png'))`. O sufixo `Template` ativa a inversão automática, e o `@2x` é carregado sozinho. Durante o download, faça `tray.setImage()` com o frame `progress-XXX` mais próximo.

---

## 12. Notas legais e de distribuição
- **Não usar** o logo do YouTube nem o nome "YouTube" no nome ou no ícone do app (marca registrada).
- Apps que baixam vídeos do YouTube violam os Termos do YouTube e **são rejeitados na Mac App Store**. Distribua por fora: **DMG assinado com Developer ID e notarizado**, com atualizações via Sparkle.
- Mostrar um aviso de uso responsável na primeira execução ("Baixe apenas conteúdo que você tem direito de baixar").
