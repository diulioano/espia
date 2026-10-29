# Research — 006 Transmitir câmera ou placa de captura

Data: 2026-10-29. Decisões da [spec](./spec.md): Q1 C (qualquer entrada de som, mas só junto da câmera), Q2 C (tela + câmera juntas fica para o futuro), Q3 B (celular transmite só câmera).

## R1 — Como capturar câmera/placa

- **Decisão**: `navigator.mediaDevices.getUserMedia({ video: { deviceId: { exact }, width, height, frameRate } })` no site e no app. O app usa o mesmo código, porque o WebView2 implementa `getUserMedia`.
- **Por quê**: placas USB/UVC (Elgato HD60 S+/X, AVerMedia Live Gamer, as genéricas "USB3 Video") e câmeras virtuais (OBS) aparecem como `videoinput`. Não precisa de nada nativo.
- **Alternativa rejeitada**: captura nativa em Rust (Media Foundation), como na 005. Não traz ganho: aqui a página **pode** listar e escolher o dispositivo.

## R2 — Listar dispositivos e permissão

- **Decisão**:
  - Antes da permissão, `enumerateDevices()` devolve os dispositivos **sem nome**. A aba Câmeras mostra "Permitir câmeras" e, no clique, chama `getUserMedia({ video: true })` uma vez, para o navegador ou o app pedir a permissão (FR-602).
  - Depois disso, `enumerateDevices()` traz os nomes, e o stream de teste é fechado na hora.
  - `navigator.permissions.query({ name: 'camera' })` diz se a permissão já foi dada. Se foi, a lista aparece direto.
  - `devicechange` atualiza a lista com o seletor aberto.
- **App desktop**: o wry/Tauri não instala um tratador de permissão, então o WebView2 mostra o próprio pedido (igual ao do Edge) e lembra a escolha no perfil do app. **Nenhuma mudança no Rust**, e o app 0.1.3 já serve: basta publicar o site.

## R3 — Prévias ao vivo

- **Decisão**: uma prévia por dispositivo, com `getUserMedia` em 320×180 a 15 fps, no máximo 4 ao mesmo tempo (FR-603). As demais aparecem com o ícone e "toque para ver", e tocar troca qual está ao vivo.
  - As prévias ficam num `<video muted playsinline>` dentro do `SourceCard`, que ganha a prop `stream`.
  - Todas param ao fechar o seletor ou ao transmitir (FR-604).
- **Câmera ocupada**: `NotReadableError` → o cartão mostra "Em uso por outro programa" e fica desabilitado.
- **Antes de transmitir**: as prévias são fechadas e o dispositivo é reaberto na qualidade escolhida, porque alguns drivers não aceitam duas resoluções ao mesmo tempo.

## R4 — Qual entrada de som (Q1: C)

- **Decisão**: uma lista com **todas** as `audioinput`, mais "Sem som".
  - **Sugestão inicial**: a entrada com o **mesmo `groupId`** do vídeo (placas e webcams com microfone embutido expõem o mesmo grupo). Se não houver, a de nome mais parecido (tokens em comum). Se não houver nenhuma parecida, "Sem som".
- **Tipo do som**:
  - **"Som de jogo"** (sem `echoCancellation`/`noiseSuppression`/`autoGainControl`, em estéreo): quando a entrada é do mesmo grupo de uma placa ou o nome tem "capture", "hdmi", "line", "elgato", "avermedia", "cam link" ou "game".
  - **"Som de voz"** (com os três ligados, em mono): quando o nome tem "mic", "microfone", "headset", "fone" ou é o dispositivo `default`/`communications`.
  - **Na dúvida**: voz, que é o mais seguro contra eco.
  - A pessoa troca por um seletor de dois botões (FR-606).
- **Aviso do Discord** (US3 cenário 4): só heurística, sem acesso a processos no site. Com uma entrada de voz escolhida, aparece sempre a dica "Se você está no Discord, a galera pode te ouvir duas vezes. Fone de ouvido evita eco."
- **AudioKind novo**: `'device'` ("com som do dispositivo"). Para quem assiste, continua "· com som".

## R5 — Qualidade

- **Decisão**: as mesmas opções de 720/1080 e 30/60, pedidas como `ideal` (a câmera entrega o que suporta). O teto por número de espectadores (`quality.ts`) continua via `applyConstraints`, que funciona com tracks de câmera. `contentHint`: `'motion'` (câmera é sempre movimento).
- **Celular** (Q3: B): começa em 720p30, sem a opção de 1080p60 (bateria/4G), usando `facingMode` (frontal/traseira) e não a lista de dispositivos, porque os nomes no celular são confusos ("camera2 1, facing back").

## R6 — Celular transmitindo

- **Decisão**: `canShare` vira `canShareScreen` (getDisplayMedia) + `canShareCamera` (getUserMedia). O botão Transmitir aparece no celular quando há câmera. O seletor no celular mostra só a aba Câmeras.
- **Tela apagada/troca de app**: o iOS/Android pausa a câmera (`mute`/`ended` no track) → o mesmo caminho de "fonte encerrada".
- **Constituição**: "assistir: qualquer navegador moderno, inclusive celular" passa a ter "transmitir câmera: celular também" (emenda junto da 1.3.0).

## R7 — Protocolo

- **Decisão**: o `live` ganha `source?: 'screen' | 'camera'` (padrão `'screen'`, compatível com clientes antigos), e o `AudioKind` ganha `'device'`. O Worker valida e repassa. Quem assiste usa `source` para o rótulo "câmera de <apelido>" (FR-607).

## R8 — Constituição 1.3.0 (MINOR)

- **Princípio I**: "Funcionalidades fora do escopo (voz, chat de texto, …)" passa a dizer que voz **como recurso próprio** continua fora, mas **o som de uma câmera transmitida (inclusive microfone) é permitido**, só enquanto a câmera estiver no ar.
- **Restrições técnicas**: "Navegadores alvo para transmitir: Chrome/Edge desktop" ganha "câmera: também celular".

## R9 — Testes

- **Vitest**:
  - funções puras em `src/lib/devices.ts` (sugestão de entrada por `groupId`/nome, voz × jogo, constraints por tipo, limite de 4 prévias);
  - o Worker aceitando `source`/`'device'`.
- **Mocks**: `pickercamperm`, `pickercams`, `pickercamnone`, `pickercamdenied`, `pickercammic`, `mobilecam`, `livecam`.
- **E2E/manual**: a webcam real e a placa precisam de hardware. Os testes no Edge usam `--use-fake-device-for-media-stream` (câmera **falsa**, que gera um padrão de teste e não captura nada real) e conferem o padrão recebido. A captura da tela continua proibida nos testes (memória do projeto).
