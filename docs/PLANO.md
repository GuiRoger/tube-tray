# TubeTray — Escopo, plano e checklist de validação

> Documento vivo do projeto. Atualize o **Status** e as caixas a cada entrega e rode o
> **Checklist de validação** antes de gerar qualquer versão.
> Repositório: <https://github.com/GuiRoger/tube-tray> · Design: [`design/DESIGN_SYSTEM.md`](../design/DESIGN_SYSTEM.md)

## Status (20/10/2026)

| Fase | Situação |
|---|---|
| 0 — Fundação | Concluída |
| 1 — Núcleo de download | Concluída |
| 2 — Bandeja e janela | Concluída, validada no Windows |
| 3 — Acabamento | Concluída (+ prévia, histórico, arquivos excluídos, ícones vermelhos) |
| 4 — Empacotamento e distribuição | Concluída: v0.1.2 publicada, links diretos funcionando, site no ar, Sponsors ativo. Falta só o teste no Mac |
| 5 — Versão 1.0 | Implementada (perfis de áudio e qualidade, atalho global, playlists, downloads simultâneos); falta validar no Windows e publicar a tag `v1.0.0` |
| 6 — Versão 1.1 | Planejada: Linux, atualização automática, instaladores assinados |

Papéis: **Gui** valida, aponta correções e passa requisitos; **Claude** implementa.

Distribuição: instaladores só pelo GitHub ([versões](https://github.com/GuiRoger/tube-tray/releases)); página do projeto em [tubetray.vercel.app](https://tubetray.vercel.app/) (projeto separado `tube-tray-landing-page`). Mac App Store e Microsoft Store rejeitam esse tipo de app.

## Visão

App de bandeja (Windows) / barra de menu (macOS) que recebe um link do YouTube e baixa o vídeo (ou só o áudio) com um clique, usando o yt-dlp. O formato padrão dá o mesmo resultado do comando:

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
- Fila com downloads simultâneos (1 a 5, padrão 2), cancelar, cancelar todos, tentar de novo, remover da lista
- Formatos (seletor Vídeo | Áudio): Compatível · MP4 (padrão, H.264), Melhor qualidade (até 4K/8K, MP4 ou MKV), M4A e MP3; a escolha vira o padrão
- Playlists: lista os vídeos (até 500) com miniatura e duração, escolha dos itens, subpasta com o nome da playlist e notificação única com o resumo
- Atalho global (padrão Ctrl+Alt+Y / ⌥⌘Y, configurável) que baixa o link copiado sem abrir a janela, com notificação de confirmação
- Progresso com %, velocidade e tempo restante; vídeo + áudio numa barra só
- Notificação do sistema ao concluir ou falhar, com título do vídeo e ícone de sucesso/falha
- Histórico salvo (últimos 100); vídeo apagado da pasta aparece riscado como "Arquivo excluído", com "Baixar de novo"
- Ajustes: abrir ao iniciar sessão, detectar link no clipboard, notificações, pasta de destino, versão/atualização do yt-dlp
- Aviso de uso responsável na primeira execução
- ffmpeg embutido; yt-dlp baixado no primeiro uso e atualizável
- Tema claro/escuro e alto contraste; atalhos Ctrl/⌘+1/2 (Vídeo/Áudio), Esc, Ctrl/⌘+O, Ctrl/⌘+, e Ctrl/⌘+Q
- Ajustes → Sobre: versão, link do código-fonte e tela de **Licenças** (TubeTray, yt-dlp, FFmpeg, Tauri, React e lista completa de componentes, com os textos das licenças)
- "Apoie o desenvolvedor" opcional (GitHub Sponsors; Pix se configurado), sem nada travado
- Landing page (projeto `tube-tray-landing-page`, Vercel) com downloads da versão mais recente, instalação, aviso de uso responsável e apoio

**Fora (por enquanto)**

- Linux (versão 1.1)
- Auto-update do app e instaladores assinados (versão 1.1)
- Canais inteiros, pausar/retomar, legendas, escolher a resolução exata

## Stack

**Tauri 2 + React + TypeScript (Vite)**, com o núcleo de download em Rust.

| Peça | Uso |
|---|---|
| `src-tauri/core` (crate Rust sem UI) | perfis, fila paralela, execução do yt-dlp, leitura de progresso, playlist, histórico, prévia, settings |
| `src-tauri/src` (app Tauri) | bandeja, janela, atalho global, comandos, notificações |
| `src/` (React) | interface, seguindo o design system |
| Plugins Tauri | single-instance, positioner, autostart, clipboard-manager, dialog, opener, global-shortcut |
| Notificação | toast WinRT (Windows) e notify-rust (macOS), para ter ícone próprio |
| ffmpeg | sidecar embutido (`npm run fetch:ffmpeg`) |
| yt-dlp | baixado do GitHub para a pasta de dados do app |

## Arquitetura

A UI só chama comandos (`enqueue`, `enqueue_many`, `profiles`, `playlist_info`, `cancel`, `cancel_all`, `retry`, `video_info`, `refresh_files`, `get_settings`, `set_settings`, `update_ytdlp`…) e escuta eventos (`jobs://update`, `tools://ytdlp-download`). Só o núcleo executa o yt-dlp.

```
tube-tray/
├─ src/                       UI React (components, store, lib, styles, dev/mock.ts)
├─ src-tauri/
│  ├─ core/src/
│  │  ├─ queue.rs             fila paralela, cancelamento, histórico, arquivos excluídos
│  │  ├─ profile.rs           formatos (MP4, melhor qualidade, M4A, MP3)
│  │  ├─ playlist.rs          vídeos da playlist (--flat-playlist)
│  │  ├─ runner.rs            monta argumentos e executa o yt-dlp
│  │  ├─ progress.rs          linhas TT_* → progresso e erros em português
│  │  ├─ files.rs             decodificação da saída, achar arquivo pelo id
│  │  ├─ metadata.rs          prévia (oEmbed)
│  │  ├─ tools.rs             baixar/atualizar yt-dlp, achar ffmpeg
│  │  ├─ youtube_url.rs       validar/normalizar links
│  │  └─ settings.rs
│  ├─ src/                    lib.rs, tray.rs, window.rs, notify.rs, quick.rs (atalho), commands.rs
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
- [x] Workflow rodado e instaladores baixados
- [x] App instalado no Windows: instalação e uso funcionando
- [ ] Teste no Mac (amiga): instalar, baixar, notificação, ícone na barra
- [x] Tela de licenças e avisos de terceiros (`THIRD_PARTY_NOTICES.md`, oferta de código-fonte do FFmpeg)
- [x] Apoio opcional no app e na página (GitHub Sponsors)
- [x] Projeto desvinculado da TwinArc (identificador `io.github.guiroger.tubetray`, com migração dos dados)
- [x] Nova landing page no projeto `tube-tray-landing-page` (design system do app, downloads do último release)
- [x] Landing page publicada na Vercel (<https://tubetray.vercel.app/>)
- [x] Repositório `GuiRoger/tube-tray` público
- [x] GitHub Sponsors habilitado na conta
- [x] Primeira versão publicada (v0.1.2 como *Latest*, publicada sozinha pelo workflow)
- [x] Links diretos `releases/latest/download/TubeTray-…` baixando o instalador
- [ ] Checklist de validação completo (Windows e Mac)

**Fase 5 — Versão 1.0**

- [x] Perfis: Compatível · MP4 (padrão), Melhor qualidade (até 4K/8K, MP4/MKV), M4A e MP3
- [x] Seletor Vídeo | Áudio + formato na janela (Ctrl/⌘+1/2); escolha vira o padrão; formato padrão nos Ajustes
- [x] Atalho global "baixar o link copiado" (Ctrl+Alt+Y / ⌥⌘Y), configurável e desligável; avisa se outro app usa as teclas
- [x] Playlists: lista com miniatura e duração, marcar/desmarcar, subpasta com o nome, resumo numa notificação só
- [x] Vídeo dentro de playlist: baixa só o vídeo, com opção "Escolher vídeos"
- [x] Downloads simultâneos (1 a 5, padrão 2), "Cancelar todos"; ícone da bandeja mostra a média
- [x] Testes: perfis com o yt-dlp real e o ffmpeg do instalador (MP3, M4A, melhor qualidade), fila paralela, playlist, atalhos
- [ ] Teste no Windows com o build do Actions (seção "Versão 1.0" do checklist)
- [ ] Tag `v1.0.0` publicada (vira *Latest* e os links do site passam a baixar a 1.0)
- [ ] Site atualizado com as novidades (projeto `tube-tray-landing-page`)

**Fase 6 — Versão 1.1 (planejada)**

- [ ] Linux (AppImage/.deb; bandeja via AppIndicator)
- [ ] Atualização automática do app (`tauri-plugin-updater`, assinatura das atualizações)
- [ ] Instalador assinado no Windows (certificado de assinatura de código) e app assinado e notarizado no Mac (Apple Developer ID)

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
- [ ] W M — Mesmo link duas vezes no mesmo formato: "Este vídeo já está na fila neste formato."
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

**Versão 1.0**

- [ ] W M — Vídeo | Áudio: Ctrl/⌘+1/2 alternam; o formato escolhido continua depois de fechar e abrir o app
- [ ] W M — Melhor qualidade: vídeo 4K baixa em resolução máxima e abre (VLC ou player do sistema)
- [ ] W M — M4A e MP3: só o áudio, arquivo toca e tem título; status "Convertendo áudio…" no fim
- [ ] W M — Mesmo vídeo em MP4 e MP3 ao mesmo tempo: os dois baixam
- [ ] W M — Playlist (`/playlist?list=`): lista aparece, desmarcar itens muda o "Baixar N vídeos", vídeos vão para a subpasta
- [ ] W M — Playlist com vídeo privado: item aparece desabilitado
- [ ] W M — Link de vídeo com `&list=`: baixa só o vídeo; "Escolher vídeos" abre a playlist
- [ ] W M — Fim da playlist: uma notificação só ("Playlist baixada", N vídeos); clique abre a pasta (W)
- [ ] W M — Downloads simultâneos = 2: dois baixam juntos, o resto "Na fila"; trocar para 3 já começa o próximo
- [ ] W M — "Cancelar todos" para tudo e não deixa `.part`
- [ ] W M — Atalho global com o app fechado na bandeja: notificação "Baixando" e o vídeo entra na fila
- [ ] W M — Atalho sem link do YouTube copiado: notificação "Nenhum link do YouTube copiado"
- [ ] W M — Atalho com link só de playlist: abre a janela com a lista
- [ ] W M — Trocar o atalho nos Ajustes; o novo funciona e o antigo não; desligar desativa
- [ ] W M — Atalho já usado por outro app: Ajustes mostram o aviso e mantêm o anterior
- [ ] W M — Menu do ícone "Colar e baixar" se comporta igual ao atalho
- [ ] W M — Ajustes → Sobre mostra v1.0.0

**Página do projeto**

- [ ] Página abre em https://tubetray.vercel.app/ (claro, escuro e celular); prévia do link no X/WhatsApp mostra a imagem nova
- [ ] Botões de download do site e links do README baixam direto o instalador da versão publicada (`releases/latest/download/TubeTray-…`)
- [ ] Nenhum texto promove baixar música ou filmes (o áudio é apresentado para podcasts, aulas e entrevistas); aviso de uso responsável visível

## Riscos

| Risco | Impacto | Mitigação |
|---|---|---|
| YouTube muda e o yt-dlp quebra | Downloads falham | Botão Atualizar yt-dlp, sem novo build do app |
| Mac bloqueia app não assinado | "App danificado" ao abrir | Assinatura ad-hoc + `xattr -cr` (README); Developer ID se for distribuir |
| SmartScreen no Windows | Aviso na 1ª execução | "Mais informações → Executar assim mesmo"; assinatura depois |
| Saída do yt-dlp em outra codificação | Títulos/caminhos errados | `--encoding utf-8`, leitura com reserva Windows-1252 e busca do arquivo pelo id |
| Muitos downloads simultâneos | YouTube limita (erro 429) | Máximo de 5; padrão 2; mensagem "Muitas requisições… aguarde" |
| Atalho global já usado por outro app | Atalho não funciona | Ajustes mostram o aviso e deixam trocar as teclas |
| AV1/VP9 em "Melhor qualidade" | Player padrão do Windows não abre | Texto do perfil avisa; "Compatível · MP4" continua o padrão |
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
- Distribuição: instaladores pelo GitHub Releases; página na Vercel
- v1.0: formato padrão continua o MP4 compatível; áudio em M4A (sem reconversão) e MP3; playlists em subpasta; 2 downloads simultâneos por padrão; atalho Ctrl+Alt+Y / ⌥⌘Y (o ⌥⌘D do design system esconde o Dock no Mac)
- Linux, auto-update e instalador assinado ficam para a v1.1

## Depois da 1.1

- Canais inteiros (com filtro por data)
- Pausar/retomar
- Legendas
- Escolher a resolução exata (1080p, 720p…)
