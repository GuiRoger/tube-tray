# Licenças de terceiros

O código do TubeTray é MIT ([LICENSE](LICENSE)). Os instaladores e o app usam os componentes abaixo, que mantêm as próprias licenças. Os textos completos aparecem no app em **Ajustes → Sobre → Licenças** e estão em [`src/assets/licenses/`](src/assets/licenses/).

| Componente | Licença | Como entra no TubeTray | Código-fonte |
|---|---|---|---|
| **FFmpeg** 6.0 (build estático) | GPL-3.0 | Embutido nos instaladores, como programa separado (`ffmpeg` ao lado do executável) | <https://ffmpeg.org/download.html> · builds de [ffmpeg-static b6.0](https://github.com/eugeneware/ffmpeg-static/releases/tag/b6.0) |
| **yt-dlp** | Unlicense (código) · GPLv3+ (executável empacotado) | **Não** vem no instalador: o app baixa o executável oficial do GitHub do projeto no primeiro uso | <https://github.com/yt-dlp/yt-dlp> |
| **Tauri** e plugins | MIT ou Apache-2.0 | Biblioteca do app | <https://github.com/tauri-apps/tauri> |
| **React** | MIT | Interface | <https://github.com/facebook/react> |

## Oferta de código-fonte (GPL)

O FFmpeg incluído é software livre sob a GPL-3.0. O código-fonte correspondente à versão distribuída está disponível em <https://ffmpeg.org/releases/> (versão 6.0). Se preferir, abra uma *issue* em <https://github.com/GuiRoger/tube-tray/issues> e enviamos o código-fonte correspondente, por pelo menos três anos a partir de cada versão publicada.

## Demais componentes

A lista completa de bibliotecas Rust e npm usadas pelo app (nome, versão, licença e repositório) fica em [`src/assets/licenses/third-party.json`](src/assets/licenses/third-party.json) e aparece no app em *Licenças → Ver outros componentes*. Para regerar depois de mudar dependências:

```bash
npm run licenses
```

---

Projeto independente, sem afiliação com o YouTube ou o Google. YouTube é marca registrada da Google LLC.
