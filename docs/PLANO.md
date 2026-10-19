# TubeTray — Escopo, plano e checklist de validação

> Documento vivo do projeto. Atualize o **Status** e as caixas a cada entrega e rode o
> **Checklist de validação** antes de gerar qualquer versão.
> Repositório: <https://github.com/GuiRoger/tube-tray> · Design: [`design/DESIGN_SYSTEM.md`](../design/DESIGN_SYSTEM.md)

## Status (19/10/2026)

| Fase | Situação |
|---|---|
| 0 — Fundação | Concluída |
| 1 — Núcleo de download | Concluída |
| 2 — Bandeja e janela | Concluída, validada no Windows |
| 3 — Acabamento | Concluída (+ prévia, histórico, arquivos excluídos, ícones vermelhos) |
| 4 — Empacotamento e distribuição | Em andamento: build no GitHub Actions, teste no Mac, landing page no ar |

Papéis: **Gui** valida, aponta correções e passa requisitos; **Claude** implementa.

Distribuição: só pelo GitHub ([versões](https://github.com/GuiRoger/tube-tray/releases) e [página do projeto](https://guiroger.github.io/tube-tray/)). Mac App Store e Microsoft Store rejeitam esse tipo de app.

## Visão

App de bandeja (Windows) / barra de menu (macOS) que recebe um link do YouTube e baixa em MP4 com um clique, usando o yt-dlp. O resultado é o mesmo do comando:

```
yt-dlp -S "vcodec:h264,res,acodec:m4a" --merge-output-format mp4 "<link>"
```

Princípios: pequeno, bonito, rápido de abrir, e com o núcleo separado da interface para crescer sem reescrever.

## Escopo

**Entregue**

- Ícone na bandeja/barra de menu com fundo vermelho e estados: parado, progresso (anel de 10 em 10%), concluído e erro não vistos
- Menu do ícone: Abrir, Colar e baixar, Abrir pasta de downloads, Ajustes, Sair
- Janela popover de 340 px ancorada ao ícone; some ao perder o foco ou com Esc
- Campo de URL preenchido pelo clipboard ao abrir; validação de link
- Prévia ao colar: miniatura, título e canal (oEmbed do YouTube)
- Fila (um download por vez), cancelar, tentar de novo, remover da lista
- Progresso com %, velocidade e tempo restante; vídeo + áudio numa barra só
- Notificação do sistema ao concluir ou falhar, com título do vídeo e ícone de sucesso/falha
- Histórico salvo (últimos 100); vídeo apagado da pasta aparece riscado como "Arquivo excluído", com "Baixar de novo"
- Ajustes: abrir ao iniciar sessão, detectar link no clipboard, notificações, pasta de destino, versão/atualização do yt-dlp
- Aviso de uso responsável na primeira execução
- ffmpeg embutido; yt-dlp baixado no primeiro uso e atualizável
- Tema claro/escuro e alto contraste; atalhos Esc, Ctrl/⌘+O, Ctrl/⌘+, e Ctrl/⌘+Q
- Ajustes → Sobre: versão, link do código-fonte e tela de **Licenças** (TubeTray, yt-dlp, FFmpeg, Tauri, React e lista completa de componentes, com os textos das licenças)
- "Apoie o desenvolvedor" opcional (GitHub Sponsors; Pix se configurado), sem nada travado
- Landing page (`site/`) no GitHub Pages com downloads da versão mais recente, instalação e aviso de uso responsável

**Fora (por enquanto)**

- Escolha Vídeo/Áudio e qualidade
- Playlists e canais
- Pausar/retomar e downloads simultâneos
- Atalho global
- Auto-update do app e instalador assinado

## Stack

**Tauri 2 + React + TypeScript (Vite)**, com o núcleo de download em Rust.

| Peça | Uso |
|---|---|
| `src-tauri/core` (crate Rust sem UI) | fila, execução do yt-dlp, leitura de progresso, histórico, prévia, settings |
| `src-tauri/src` (app Tauri) | bandeja, janela, comandos, notificações |
| `src/` (React) | interface, seguindo o design system |
| Plugins Tauri | single-instance, positioner, autostart, clipboard-manager, dialog, opener |
| Notificação | toast WinRT (Windows) e notify-rust (macOS), para ter ícone próprio |
| ffmpeg | sidecar embutido (`npm run fetch:ffmpeg`) |
| yt-dlp | baixado do GitHub para a pasta de dados do app |

## Arquitetura

A UI só chama comandos (`enqueue`, `cancel`, `retry`, `video_info`, `refresh_files`, `get_settings`, `set_settings`, `update_ytdlp`…) e escuta eventos (`jobs://update`, `tools://ytdlp-download`). Só o núcleo executa o yt-dlp.

```
tube-tray/
├─ src/                       UI React (components, store, lib, styles, dev/mock.ts)
├─ src-tauri/
│  ├─ core/src/
│  │  ├─ queue.rs             fila, cancelamento, histórico, arquivos excluídos
│  │  ├─ runner.rs            monta argumentos e executa o yt-dlp
│  │  ├─ progress.rs          linhas TT_* → progresso e erros em português
│  │  ├─ files.rs             decodificação da saída, achar arquivo pelo id
│  │  ├─ metadata.rs          prévia (oEmbed)
│  │  ├─ tools.rs             baixar/atualizar yt-dlp, achar ffmpeg
│  │  ├─ youtube_url.rs       validar/normalizar links
│  │  └─ settings.rs, profile.rs
│  ├─ src/                    lib.rs, tray.rs, window.rs, notify.rs, commands.rs
│  └─ icons/                  app, bandeja (windows/ e macos/), notificação
├─ design/                    design system e gerador de ícones
├─ docs/PLANO.md              este documento
└─ .github/workflows/build.yml
```

Arquivos do usuário: `%APPDATA%\io.github.guiroger.tubetray\` (Windows) ou `~/Library/Application Support/io.github.guiroger.tubetray/` (Mac) — `settings.json`, `history.json`, `bin/yt-dlp`.

## Plano de execução

**Fase 0 — Fundação**

- [x] Projeto Tauri 2 + React + TS + Vite
- [x] App só de bandeja (fora do Dock no macOS)
- [x] Script que baixa o ffmpeg por plataforma
- [x] Testes Rust e TypeScript rodando

**Fase 1 — Núcleo de download**

- [x] yt-dlp baixado no 1º uso + ffmpeg sidecar
- [x] Runner com o comando original + protocolo de progresso
- [x] Parser de progresso, velocidade, ETA e título (com testes)
- [x] Fila com cancelamento (encerra yt-dlp e ffmpeg, apaga temporários)
- [x] Validação de link (youtube.com, youtu.be, shorts, live)

**Fase 2 — Bandeja e janela**

- [x] Ícone com menu e estados
- [x] Janela popover ancorada ao ícone
- [x] Link do clipboard ao abrir
- [x] Lista com progresso, velocidade, ETA e cancelar

**Fase 3 — Acabamento**

- [x] Notificações com ícone de sucesso/falha
- [x] Ajustes (pasta, iniciar com o sistema, clipboard, notificações)
- [x] Atualizar yt-dlp
- [x] Mensagens de erro em português
- [x] Prévia do vídeo ao colar
- [x] Histórico e detecção de arquivos excluídos
- [x] Correção: títulos com acento no Windows (vídeos marcados como excluídos)

**Fase 4 — Empacotamento**

- [x] Configuração de build por plataforma (NSIS no Windows; .app/.dmg com assinatura ad-hoc no Mac)
- [x] Workflow do GitHub Actions (testes + instaladores Windows, Mac Apple Silicon e Mac Intel)
- [x] Código enviado ao GitHub (`git push` a cada entrega)
- [ ] Workflow rodado e instaladores baixados
- [ ] App instalado no Windows: notificação como "TubeTray" e clique abrindo o arquivo
- [ ] Teste no Mac (amiga): instalar, baixar, notificação, ícone na barra
- [x] Tela de licenças e avisos de terceiros (`THIRD_PARTY_NOTICES.md`, oferta de código-fonte do FFmpeg)
- [x] Apoio opcional no app e na página (GitHub Sponsors)
- [x] Projeto desvinculado da TwinArc (identificador `io.github.guiroger.tubetray`, com migração dos dados)
- [x] Landing page em `site/` + workflow do GitHub Pages
- [ ] GitHub Pages ativado (*Settings → Pages → Source: GitHub Actions*)
- [ ] GitHub Sponsors habilitado na conta (ou chave Pix configurada)
- [ ] Primeira versão publicada (rascunho do release → *Publish*)
- [ ] Checklist de validação completo

## Checklist de validação

Rode antes de cada versão. Marque Windows (W) e Mac (M).

**Automático**

- [ ] `npm test` passa
- [ ] `npx tsc -b` sem erros
- [ ] `npm run test:core` passa
- [ ] Workflow do GitHub Actions verde

**Instalação e início**

- [ ] W M — Instalador abre e instala sem erro (Mac: ver "Instalar no Mac" no README)
- [ ] W M — Ícone vermelho aparece na bandeja/barra de menu; não aparece na barra de tarefas/Dock
- [ ] W M — Abrir de novo pelo atalho não cria segunda instância (só mostra a janela)
- [ ] W M — Primeiro download baixa o yt-dlp sozinho ("Instalando o yt-dlp…")

**Janela**

- [ ] W M — Clique esquerdo no ícone abre/fecha a janela junto ao ícone
- [ ] W M — Clique direito mostra o menu (Abrir, Colar e baixar, Pasta, Ajustes, Sair)
- [ ] W M — Clicar fora ou Esc fecha a janela; o app continua na bandeja
- [ ] W M — Tema claro e escuro do sistema aplicados

**Download**

- [ ] W M — Link copiado aparece no campo ao abrir a janela
- [ ] W M — Prévia mostra miniatura, título e canal
- [ ] W M — Link inválido mostra "Esse link não parece ser do YouTube." e desabilita Baixar
- [ ] W M — Vídeo normal baixa em MP4 que abre no player padrão
- [ ] W M — Título com acento/cedilha: nome do arquivo e título na lista corretos
- [ ] W M — Short (`/shorts/`) baixa
- [ ] W M — Vídeo longo (> 30 min): progresso, velocidade e tempo restante fazem sentido
- [ ] W M — Dois links seguidos: o segundo fica "Na fila" e começa depois
- [ ] W M — Mesmo link duas vezes: "Este vídeo já está na fila."
- [ ] W M — Cancelar para o download e não deixa `.part` na pasta
- [ ] W M — Vídeo privado/indisponível: falha com mensagem em português e "Tentar de novo"

**Depois do download**

- [ ] W M — Notificação com título do vídeo e ícone verde (falha: ícone vermelho)
- [ ] W — Clique na notificação de sucesso abre o Explorer com o arquivo selecionado
- [ ] W M — Ícone da bandeja mostra o ponto de concluído até abrir a janela
- [ ] W M — Mostrar na pasta e Abrir funcionam; duplo clique abre o vídeo
- [ ] W M — Fechar e abrir o app: histórico continua, sem itens riscados indevidamente
- [ ] W M — Apagar um vídeo da pasta: aparece riscado como "Arquivo excluído"; "Baixar de novo" funciona

**Ajustes**

- [ ] W M — Trocar a pasta de destino; próximo download vai para a nova pasta
- [ ] W M — "Abrir ao iniciar sessão" liga/desliga de verdade (reiniciar o PC)
- [ ] W M — Desligar notificações: nenhuma notificação no fim
- [ ] W M — Atualizar yt-dlp mostra a versão nova
- [ ] W M — Sobre mostra a versão certa; Licenças abre os textos e "Código-fonte" abre o navegador
- [ ] W M — Apoie: botão do GitHub Sponsors abre a página certa (e Pix copia a chave, se configurado)
- [ ] W — Atualizando de uma versão com o identificador antigo: ajustes e histórico continuam lá

**Página do projeto**

- [ ] Página abre em https://guiroger.github.io/tube-tray/ (claro, escuro e celular)
- [ ] Botões de download baixam os instaladores da versão publicada
- [ ] Nenhum texto promove baixar música ou filmes; aviso de uso responsável visível

## Riscos

| Risco | Impacto | Mitigação |
|---|---|---|
| YouTube muda e o yt-dlp quebra | Downloads falham | Botão Atualizar yt-dlp, sem novo build do app |
| Mac bloqueia app não assinado | "App danificado" ao abrir | Assinatura ad-hoc + `xattr -cr` (README); Developer ID se for distribuir |
| SmartScreen no Windows | Aviso na 1ª execução | "Mais informações → Executar assim mesmo"; assinatura depois |
| Saída do yt-dlp em outra codificação | Títulos/caminhos errados | `--encoding utf-8`, leitura com reserva Windows-1252 e busca do arquivo pelo id |
| WebView diferente por sistema | Pequenas diferenças visuais | CSS simples; checklist nas duas plataformas |
| Plataforma de pagamento bloquear a conta de apoio | Perda do canal de doações | Apoio opcional e sem contrapartida; GitHub Sponsors/Pix; sem ligação com a TwinArc |
| Denúncia por violação de direitos autorais | Remoção do repositório/release | Aviso de uso responsável; nada de marketing para música/filmes; sem marca do YouTube |

## Decisões tomadas

- Nome: **TubeTray**
- Pasta padrão: `Downloads/TubeTray`
- Repositório: `GuiRoger/tube-tray`
- Ícones da bandeja com fundo vermelho também no Mac (o design system pedia monocromático)
- Ajustes dentro da própria janela, não numa janela separada
- App gratuito e de código aberto (MIT); apoio só por doação opcional, nunca cobrança pelo download
- Sem ligação com a TwinArc Systems (identificador, autoria e páginas em nome de GuiRoger)
- Distribuição só pelo GitHub (releases + GitHub Pages)

## Depois do MVP

- Perfis: "Só áudio M4A/MP3", "Melhor qualidade"
- Atalho global que baixa o link do clipboard
- Playlists com seleção de itens
- Downloads paralelos
- Linux
- Auto-update do app e instalador assinado
