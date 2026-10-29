# Research: Instalar o app pela tela inicial

## R1. Link fixo para o instalador mais novo

- **Decision**: `https://github.com/DiulianoVargas/espia/releases/latest/download/Espia-setup.exe`. O GitHub redireciona `latest/download/<nome>` para o asset com esse nome na Release mais nova. O `tauri-action` gera `Espia_<versão>_x64-setup.exe`, então o `release.yml` ganha um passo que copia o arquivo como `Espia-setup.exe` e sobe na mesma Release (`gh release upload --clobber`). Na `v0.1.4`, já publicada, o asset é subido uma vez à mão.
- **Rationale**: um endereço que nunca muda, sem código no Worker, sem a API do GitHub (que tem limite de 60 consultas/hora por IP, e os IPs da Cloudflare são compartilhados) e sem segredo.
- **Fallback (FR-802)**: com o passo no workflow, toda Release tem o asset. O site não consegue saber de antemão se o GitHub vai responder 404 (download cross-origin), então a dica depois do clique traz também o link "Não baixou? Abra a página de versões".
- **Alternatives**:
  - rota `/baixar` no Worker consultando a API do GitHub (limite de consultas e mais um ponto de falha);
  - pôr o `.exe` nos assets do Worker (limite de 25 MiB por arquivo e um deploy do site a cada versão do app).

## R2. Quando mostrar o convite

- **Decision**: `appInviteVisible({ platform, ua, mobile, inApp, dismissed })`, uma função pura:
  - `inApp` (`'__TAURI_INTERNALS__' in window`) → não;
  - `dismissed` → não;
  - `navigator.userAgentData?.platform === 'Windows'` com `mobile === false` → sim;
  - sem `userAgentData` (Firefox): `/Windows NT/.test(ua)` e sem `/Mobile|Android/` → sim;
  - qualquer outro caso → não.
- **Rationale**: `userAgentData` (Chrome/Edge) não mente sobre o sistema. O UA serve de reserva. Na dúvida, não mostra (edge case da spec).
- **Windows ARM**: aparece igual (o x64 roda emulado).

## R3. Textos do instalador sem modelo próprio

- **Decision**: `bundle.windows.nsis.installerHooks = "nsis/hooks.nsh"`. O modelo `installer.nsi` do Tauri 2.12.4 faz `!include "{{installer_hooks}}"` na linha 36, antes de `MUI_PAGE_WELCOME` (171) e `MUI_PAGE_FINISH` (443). Os `!define` do MUI no `hooks.nsh` valem para essas páginas:
  - `MUI_WELCOMEPAGE_TITLE` "Bora instalar o Espia";
  - `MUI_WELCOMEPAGE_TEXT` (o que o app traz e "não precisa de administrador");
  - `MUI_FINISHPAGE_TITLE` "Pronto!";
  - `MUI_FINISHPAGE_TEXT`;
  - `MUI_FINISHPAGE_RUN_TEXT` "Abrir o Espia".
- **Frases do Tauri** (app aberto, reinstalar, WebView2): `customLanguageFiles: { "PortugueseBR": "nsis/PortugueseBR.nsh" }`, com o arquivo oficial copiado e reescrito em tom informal (corrige também "primeirl" e "executare" do original).
- **"Abrir o Espia" marcado**: o modelo já define `MUI_FINISHPAGE_RUN` (marcado por padrão). Nada a fazer além do texto.
- **App aberto ao atualizar** (US2-3): o modelo já detecta o app aberto e oferece fechar (`appRunningOkKill`). Só muda o texto.
- **Verificado (2026-10-29)**: os `!define` pegaram (tela de boas-vindas conferida). Cuidados: o Tauri já grava o arquivo de idioma com BOM, então o `PortugueseBR.nsh` vai **sem** BOM (com BOM fica duplicado e o makensis quebra na linha 1); o `hooks.nsh` é incluído direto e vai **com** BOM (UTF-8). O botão do Windows em português é "Próximo", não "Avançar".
- **Alternatives**: `nsis.template` com uma cópia inteira do `installer.nsi` (mais controle, mas fica preso à versão do Tauri). Fica como reserva se a ordem mudar.

## R4. Imagens do instalador

- **Decision**: geradas com Pillow a partir de `src-tauri/icons/icon.png` (o castor) e das cores de `src/ui/tokens.css` (fundo escuro do Espia, destaque `--color-accent`):
  - `sidebar.bmp` 164×314 (boas-vindas e fim): fundo no gradiente escuro do site, castor grande, "Espia";
  - `header.bmp` 150×57 (páginas do meio): castor pequeno à direita, fundo claro, porque o texto do NSIS no topo é preto.
- **Formato**: BMP 24 bits, sem alfa (o NSIS não aceita alfa no BMP). Versionadas no repositório.
- **Escala alta**: o NSIS estica o BMP em 150%. Aceitável; o castor é grande o bastante.

## R5. Ícone da aba (US4)

- **Decision**: em `public/`:
  - `favicon.ico` (16, 32, 48), gerado do `icon.png`;
  - `icon-192.png`;
  - `apple-touch-icon.png` (180, fundo cheio, sem transparência, como o iOS pede).

  No `index.html`: `<link rel="icon" href="/favicon.ico" sizes="any">`, `<link rel="icon" type="image/png" sizes="192x192" href="/icon-192.png">`, `<link rel="apple-touch-icon" href="/apple-touch-icon.png">` e `<meta name="theme-color">` com o fundo do Espia.
- **Rationale**: hoje `/favicon.ico` cai no SPA (`not_found_handling`) e devolve HTML. Com o arquivo em `public/`, o Worker serve o asset antes do fallback. A CSP `img-src 'self'` já cobre.
- **Alternatives**: SVG do mascote como favicon (o Safari não usa SVG na aba; o PNG/ICO cobre todos).

## R6. Onde fica a dica do SmartScreen

- **Decision**: depois de "Baixar o app", o bloco troca o texto por: "Baixando… Se o Windows avisar que protegeu o computador, clique em **Mais informações → Executar assim mesmo**." com o link "Não baixou? Abra a página de versões". O estado `hint` dura até recarregar a página.
- **Rationale**: o mesmo texto da Release (003), no momento em que a pessoa precisa.
