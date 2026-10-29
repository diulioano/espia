# Relatório — 2026-10-29 · Espia: transmitir câmera ou placa de captura (006)

## O que foi pedido

Transmitir a câmera ou outras entradas (placa de captura). Decisões:
- **Q1: C**: qualquer entrada de som, inclusive microfone, só junto da câmera;
- **Q2: C**: tela + câmera juntas fica para o futuro;
- **Q3: B**: celular transmite a câmera.

## Como ficou

- **Aba "Câmeras"** no seletor (site, app e celular), ao lado de Janelas e Telas inteiras.
  - **Permissão**: só no clique em "Permitir câmeras". Se o navegador ou o app já deu a permissão, a lista aparece direto.
  - **Cartões**: um por dispositivo (webcam, placa, câmera virtual do OBS), com **prévia ao vivo** (até 4 ao mesmo tempo). As prévias desligam ao trocar de aba, ao fechar o seletor e ao transmitir.
  - **Avisos**: "Em uso por outro programa", "Câmera bloqueada" e "Nenhuma câmera ou placa encontrada".
- **Som**: uma lista com **todas** as entradas + "Sem som", e o seletor **Som de voz | Som de jogo**.
  - **Sugestão**: a entrada do mesmo aparelho que o vídeo (a placa e o som dela costumam ser o mesmo dispositivo). Senão, a de nome parecido.
  - **Som de jogo** vai cru (sem cancelamento de eco, redução de ruído ou ganho automático, em estéreo).
  - **Som de voz** vai com os três (mono). Na dúvida, voz.
  - Com voz, aparece a dica "Se você está no Discord, a galera pode te ouvir duas vezes. Fone de ouvido evita eco."
  - O som só existe junto da câmera: parar a câmera para tudo.
- **Celular**: o botão Transmitir abre só a aba Câmeras, com **Frontal | Traseira**, em 720p30 fixo e o aviso de internet/bateria. Durante a transmissão, o botão vira "Parar câmera". Tela apagada por mais de 3 s ou troca de app encerra a transmissão.
- **Quem assiste** vê "câmera de <apelido>" no tile. Quem transmite vê "câmera · <dispositivo>" no card ao vivo.
- **Lembra** a última câmera, a entrada de som e o tipo de som (pelo nome).
- **Protocolo**: `live` com `source: 'screen' | 'camera'` (cliente antigo → `'screen'`) e `AudioKind` `'device'`. O Worker valida.
- **Constituição 1.3.0**: voz como recurso próprio continua fora. O som de uma câmera transmitida é permitido, e o celular transmite câmera.
- **App desktop**: nada mudou no Rust. O app 0.1.3 já pega a câmera (o WebView2 mostra o próprio pedido de permissão). Basta publicar o site.

## Testes

- `npm run check`: 0 erros. `npm test`: **102 passando**. Os novos cobrem:
  - `devices.ts`: placa × webcam, voz × jogo, sugestão por grupo/nome, constraints, limite de 4 prévias;
  - o Worker com `source`/`'device'`, cliente antigo e `source` inválido.
- Mocks conferidos no Edge headless: `pickercamperm`, `pickercams`, `pickercammic`, `pickercamnone`, `pickercamdenied`, `livecam` e celular `camera`.
- **E2E com a câmera falsa do Edge (T628): 10/10 ok** (depois da correção abaixo): a aba lista e mostra a prévia; A transmite 1 faixa de vídeo; 'Som de jogo' com echoCancellation false e 'Som de voz' com true; B vê 'câmera de Rafa', recebe a imagem e 'com som'; fechar o seletor tira as prévias.

## Bug achado no teste manual (2026-10-29)

A pessoa usuária tinha uma C920 conectada e ela **não aparecia** na aba. Causa: o site mandava o cabeçalho `Permissions-Policy: camera=(), microphone=()` (da época em que o Espia só transmitia tela), que **proíbe** câmera e microfone na página. O navegador e o app negavam antes de perguntar. Era também o motivo do 'Permission denied' no E2E. Corrigido para `camera=(self), microphone=(self)` em `src/worker/index.ts` e `public/_headers` (registrado em `specs/001…/security.md`).

No caminho, a aba também passou a explicar quando a câmera não abre: em uso por outro programa, não encontrada ou outro erro, com o botão 'Tentar de novo'. Antes ela só dizia 'Nenhuma câmera'.

Outros dois bugs achados pelo E2E e corrigidos:
1. Um laço infinito de efeito ao trocar de aba.
2. Câmeras sem nome até a página abrir a câmera uma vez: agora a aba abre e fecha a câmera uma vez, no clique.

## ⚠️ O que precisa de você

1. ~~**Publicar o site de novo**~~ — feito em 2026-10-29 (versão do Worker `59740736`, cabeçalho corrigido no ar).
2. **Roteiro C1–C7** (`quickstart.md`): webcam no Chrome e no app, placa + console com "Som de jogo", duas pessoas com microfone pelas caixas (eco), celular, câmera ocupada e cabo desplugado.
3. **Commit**: não fiz commit da 005 nem da 006. Me diga se é para fazer (e se é para criar a tag `v0.1.3`).
