# TubeTray

App de bandeja (Windows) / barra de menu (macOS) que baixa vídeos do YouTube em MP4 com um clique, usando o **yt-dlp**. O resultado é o mesmo do comando:

```
yt-dlp -S "vcodec:h264,res,acodec:m4a" --merge-output-format mp4 "<link>"
```

Stack: **Tauri 2** (núcleo em Rust) + **React/TypeScript** (Vite). Visual conforme `design/DESIGN_SYSTEM.md`.

## Pré-requisitos (uma vez)

| | Windows | macOS |
|---|---|---|
| Rust | [rustup](https://rustup.rs) (toolchain `stable-msvc`) | `curl https://sh.rustup.rs -sSf \| sh` |
| Compilador | **Visual Studio Build Tools** com "Desenvolvimento para desktop com C++" | `xcode-select --install` |
| WebView | WebView2 (já vem no Windows 11) | nativo |
| Node.js | 20 ou mais | 20 ou mais |

Detalhes: <https://v2.tauri.app/start/prerequisites/>

## Rodar

```bash
npm install
npm run fetch:ffmpeg      # baixa o ffmpeg da sua plataforma para src-tauri/binaries/
npm run tauri dev         # abre o app (ícone aparece na bandeja)
```

O **yt-dlp** não vem no repositório: o app baixa a versão mais recente do GitHub no primeiro download e guarda na pasta de dados (`%APPDATA%\com.twinarc.tubetray\bin` / `~/Library/Application Support/com.twinarc.tubetray/bin`). Em *Ajustes → Avançado* há o botão **Atualizar**.

Ver só a interface no navegador, sem Rust (dados simulados):

```bash
npm run dev:mock          # http://localhost:1420/?s=list | empty | notice | invalid | settings | preview | preview-loading | preview-error
```

## Gerar instalador

```bash
npm run tauri build       # Windows: .exe (NSIS) · macOS: .app/.dmg
```

O workflow `.github/workflows/build.yml` gera os instaladores de Windows e macOS (Apple Silicon e Intel). Ele roda a cada tag `v*` ou manualmente em *Actions → build → Run workflow*; os instaladores ficam em *Artifacts* no fim da execução.

### Instalar no Mac (app sem assinatura da Apple)

1. Use o `.dmg` certo: **aarch64** para Mac com chip Apple (M1/M2/M3/M4), **x86_64** para Mac Intel ( → Sobre este Mac).
2. Abra o `.dmg` e arraste o TubeTray para **Aplicativos**.
3. No Terminal, rode uma vez: `xattr -cr /Applications/TubeTray.app`
   (sem isso o macOS diz que o app "está danificado", porque ele não tem assinatura da Apple).
4. Abra o TubeTray: o ícone vermelho aparece na barra de menu, perto do relógio. Na primeira notificação, permita notificações para o app.

### Instalar no Windows

Rode o `TubeTray_x.y.z_x64-setup.exe`. Se o SmartScreen avisar, clique em *Mais informações → Executar assim mesmo* (o instalador ainda não tem assinatura de código).

O plano do projeto e o **checklist de validação** de cada versão ficam em [`docs/PLANO.md`](docs/PLANO.md).

## Testes

```bash
npm test                  # formatação (vitest)
npm run test:core         # núcleo Rust: URL, protocolo de progresso, fila, settings
```

Testes com o yt-dlp real contra um servidor HTTP local:

```bash
TT_YTDLP=/caminho/yt-dlp TT_CLIP_URL=http://127.0.0.1:8765/clip.mp4 cargo test --manifest-path src-tauri/core/Cargo.toml --test real_ytdlp
```

## Estrutura

```
src/                      UI React
  components/             Header, UrlForm, DownloadList/Row, Footer, SettingsView
  styles/tokens.css       tokens do design system (claro/escuro/alto contraste)
  assets/icons/           ícones de UI do design system
  lib/tauri.ts            comandos e eventos tipados
  dev/mock.ts             simulação do núcleo para `dev:mock`
src-tauri/
  core/                   crate sem UI: fila, runner do yt-dlp, progresso, settings, tools
  src/                    app Tauri: bandeja, janela popover, comandos
  icons/                  ícone do app + ícones da bandeja (macos/ e windows/{white,black}/)
scripts/fetch-ffmpeg.mjs  baixa o ffmpeg por target
design/                   design system e gerador de ícones
```

## Como funciona

- O núcleo chama o yt-dlp com o perfil acima e pede linhas próprias de progresso (`--progress-template`, `--print`), em vez de interpretar a saída de texto. Vídeo e áudio viram uma barra só.
- Um download por vez; os outros ficam na fila. Cancelar encerra o yt-dlp e o ffmpeg e apaga os temporários.
- Arquivos saem como `Título [id].mp4` na pasta escolhida (padrão `Downloads/TubeTray`).
- Ao colar um link, aparece a prévia com miniatura, título e canal (oEmbed do YouTube, sem chave de API).
- O ícone da bandeja (fundo vermelho) mostra o progresso (anel de 10 em 10%) e um aviso de concluído/erro até a janela ser aberta.
- Cada download que termina gera uma notificação do sistema com o título do vídeo e um ícone de sucesso ou falha. No Windows, clicar na notificação de sucesso mostra o arquivo no Explorer.
- O histórico dos downloads fica em `history.json` (últimos 100). Se um vídeo baixado for apagado da pasta, ele aparece riscado como "Arquivo excluído", com a opção de baixar de novo.

## Fora do MVP (próximos passos)

Escolha Vídeo/Áudio e qualidade, playlists, pausar/retomar, downloads simultâneos, atalho global, auto-update do app, assinatura de código.
