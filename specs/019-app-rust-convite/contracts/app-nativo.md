# Contrato: app nativo no Windows (R9, R10, R16, R17)

Vale para `Espia.exe`, o binário da crate `app/crates/espia/`. Não há motor de página, nem ponte com página, nem comando novo no app de antes (que fica congelado em `src-tauri/`).

## Linha de comando

```text
Espia.exe [<link>] [--mock [<tela>]] [--scene <cena>]
```

Variável de ambiente `ESPIA_SERVER` (exemplo `http://127.0.0.1:8787`): troca o endereço base do servidor. Só vale em build de depuração (debug). Build de release usa sempre `https://espia.diuliano.com`. O `Origin` que o app manda é sempre a origem do endereço base em uso (ver `contracts/sinalizacao-nativa.md`).

| Opção ou argumento | Efeito |
|---|---|
| sem nada | Abre a tela inicial. |
| `<link>` | Abre a sala do link (formas aceitas abaixo). O Windows passa o link de `espia://` como argumento. |
| `--mock` | Abre o índice das telas de dados falsos. |
| `--mock <tela>` | Abre a tela com a cena padrão dela. Telas: `home`, `join`, `room`, `notfound`, `offline`. |
| `--mock <tela> --scene <cena>` | Abre a tela na cena pedida. Ver `contracts/dados-falsos.md`. |

O link vale como no `links::room_path_from_args` de hoje: o primeiro argumento, pulando o caminho do executável, que for um link de sala válido. Os argumentos que não são link são ignorados. Com `<link>` e `--mock` juntos, vale o mock e o link é ignorado. Com `--mock` e uma tela ou cena que não existe, o app abre o índice do mock com uma linha no alto dizendo qual nome não existe.

## Instância única e repasse do link

- Só uma instância do app roda por pessoa (mutex nomeado `Local\dev.espia.app.instance`). A primeira instância cria o named pipe `\\.\pipe\dev.espia.app.link`.
- Abrir o app de novo, ou clicar num link `espia://`, com o app já aberto: a segunda instância manda o link pelo named pipe à primeira e termina. A primeira traz a janela para a frente. Com link, abre a sala dele; sem link, só traz a janela. Se o app está escondido na bandeja, a janela volta.
- Transmitindo e trocando de sala por link: o app pergunta antes (T071).
- App nativo e app de antes instalados juntos: só um fica instalado (mesmo identificador `dev.espia.app` e mesmo nome "Espia", instalação por cima) e só uma janela abre. O `espia://` abre o que estiver instalado.

## Formas de link aceitas

Regras de `src-tauri/src/links.rs` (`room_path`), portadas sem mudar:

| Forma | Exemplo |
|---|---|
| `espia://s/<código>` | `espia://s/k7f-q2x` (o `s` é o "host" do endereço; o código vem depois da barra) |
| `https://espia.diuliano.com/s/<código>` | endereço oficial |
| `https://espia.espia.workers.dev/s/<código>` | endereço antigo: links que já circulam continuam abrindo (FR-002) |

- O código passa para minúsculas antes de validar. Barra no fim é aceita. Texto depois do código em `https` (`?x=1`) é ignorado.
- Código válido: 7 caracteres, com `-` na posição 4 e os outros seis no alfabeto `abcdefghjkmnpqrstuvwxyz23456789` (sem `0`, `1`, `i`, `l`, `o`). É o `ROOM_ID_RE` do site.
- Recusados (nada abre): `espia://qualquer-coisa`, `espia://s/`, código curto ou com letra fora do alfabeto, `espia://x/<código>`, outro host, `http://` (sem `s`), `https://espia.diuliano.com/<código>` (sem `/s/`), código solto e texto vazio.
- A navegação interna da janela (`links::is_internal`) não é portada, porque o app nativo não carrega página. Qualquer link `http` ou `https` clicado numa tela abre no navegador padrão (FR-012). As regras de `room_path` (os dois hosts aceitos, `espia://s/<código>`) são portadas.

Link ou código colado no campo da tela inicial segue as regras de `parseRoomId` (`src/lib/room-id.ts`): link inteiro, código com ou sem hífen, com espaços. É outra regra, do site, e continua valendo ali.

## Arquivos em `%APPDATA%\dev.espia.app\`

A mesma pasta do app de antes (R9). Escrita atômica: grava num arquivo temporário e renomeia. Nada disso vai ao servidor.

| Arquivo | Para que serve | Modelo |
|---|---|---|
| `device.json` | Apelido, últimas escolhas de transmissão, volumes, Suas salas, chaves de dono, sessões, marca de importação | `data-model.md` |
| `tray.json` | Fechar no X e aviso da primeira vez. Mesmo formato de hoje | `data-model.md` |
| `window.json` | Posição e tamanho da janela | `data-model.md` |
| `resume.json` | Voltar para a sala depois de atualizar | `data-model.md`, `contracts/atualizacao.md` |

Arquivo ausente ou ilegível: padrão, sem aviso. A pasta dos dados do motor de página do app de antes (`%LOCALAPPDATA%\dev.espia.app\EBWebView\`) é outra e só é lida na importação.

## Importar os dados do app de antes (R10)

Uma vez, na primeira abertura (`importedFromWebview` ainda `false`).

| Item | Valor |
|---|---|
| Origem dos dados | `localStorage` do motor de página: `%LOCALAPPDATA%\dev.espia.app\EBWebView\Default\Local Storage\leveldb` |
| Como lê | A partir de uma cópia da pasta, com a crate `rusty-leveldb`. A pasta original não é aberta nem mexida |
| Origens lidas | `https://espia.espia.workers.dev` e `https://espia.diuliano.com` |
| Chaves lidas | As `espia:*` (lista no `data-model.md`) |
| Junção | As regras de junção do `data-model.md` (as mesmas do `#mv=`): só acrescenta, `rooms` unida por código (no mesmo código vale a entrada de maior `lastUsed`, com o nome dela) e cortada em 5, chave de dono nunca descartada |
| Depois | Grava `importedFromWebview: true` em `device.json` |
| Pasta do app de antes | Não é apagada, para ele continuar funcionando se a pessoa voltar |
| `tray.json` e posição da janela | Reaproveita o `tray.json`; converte a posição da janela do app de antes (T085); origem `%APPDATA%\dev.espia.app\.window-state.json`, formato não confirmado, e se não der para ler vale o padrão |

Formato das chaves do Chromium no LevelDB: não confirmado até o gate G3. O gate G3 (`app/spikes/webview-data/`) decide. Plano B: exportação manual pelo site, que exige replanejar.

**Se a importação falha** (banco corrompido, formato diferente, qualquer erro com a pasta presente): o app abre com o que tem e não diz nada. Não mostra erro, não trava a abertura. Tenta de novo a cada abertura enquanto `importedFromWebview` for `false` e a pasta existir. Se a pasta não existe, grava `importedFromWebview: true`.

## Bandeja (R16)

Mesmo comportamento do `specs/007-fechar-para-bandeja/contracts/tray.md`, agora feito direto pelo app, sem ponte com página. Regras do `src-tauri/src/tray.rs`, portadas.

Itens do menu, em ordem:

| Item | Aparece | Efeito |
|---|---|---|
| Abrir o Espia | sempre | Traz a janela de volta |
| Parar transmissão | só ao vivo | Para a transmissão como o botão "Parar" (manda `unlive` à sala) |
| (separador) | sempre | |
| Fechar no X | sempre, com marca | Liga e desliga `closeOnX` em `tray.json` |
| (separador) | sempre | |
| Sair | sempre | Sai de verdade; ao vivo, pergunta antes |

Estados:

| Estado | Ícone | Dica (tooltip) |
|---|---|---|
| fora do ar | `tray.png` | `Espia` |
| ao vivo | `tray-live.png` | `Espia · ao vivo` |

- Clique com o botão esquerdo no ícone traz a janela de volta. O menu abre só com o botão direito.
- O X da janela: com `closeOnX = false` (padrão), esconde a janela, mesmo ao vivo, e o app continua transmitindo. Com `closeOnX = true`, fecha: ao vivo, pergunta antes.
- "Sair" ao vivo pergunta: "Você está transmitindo. Sair mesmo assim?" (título "Espia", botão padrão "Não").
- Primeira vez que o X esconde a janela (`trayNoticeShown = false`): aviso do Windows com título "O Espia continua aberto" e texto "Ele fica aqui perto do relógio (às vezes na setinha ^). Pra sair de vez, clique com o botão direito no ícone e escolha Sair." Depois grava `trayNoticeShown = true`.
- Soltar o motor de vídeo e as telas quando escondido fora de sala (US6): tarefa T087.
- A barra "está compartilhando" não existe, porque não há motor de página (R16, FR-012).

## O que o app pode acessar no computador

Lista fechada da constituição (4.0.0), "Restrições técnicas":

> O que o app nativo pode acessar no computador é só o que transmitir exige: lista de telas e janelas, captura de imagem e som, entradas de câmera e de som, bandeja, notificações, registro do link `espia://`, pasta de dados própria, e rede até o servidor e até as pessoas da sala. Qualquer outro acesso (arquivos da pessoa, teclado ou mouse, controle remoto) exige emenda.

Leitura deste contrato:

| Acesso | Dentro da lista | Onde |
|---|---|---|
| Telas, janelas, miniaturas | lista de telas e janelas | `espia-capture` |
| Imagem e som capturados | captura de imagem e som | `espia-capture` |
| Câmera, placa de captura, entradas de som | entradas de câmera e de som | `espia-capture` |
| Bandeja e avisos do Windows | bandeja, notificações | `espia-shell` |
| `espia://` | registro do link | instalador |
| `%APPDATA%\dev.espia.app\` | pasta de dados própria | `espia-shell` |
| Servidor (HTTP e WebSocket) e pessoas da sala (WebRTC) | rede até o servidor e até as pessoas da sala | `espia-net`, `espia-media` |
| Download da atualização | rede até o servidor (mesmo endereço oficial) | `espia-shell` |

Pontos a conferir na revisão de segurança (T105): a leitura de `%LOCALAPPDATA%\dev.espia.app\EBWebView\` na importação usa o mesmo identificador do app, e este contrato a trata como pasta de dados própria. A constituição não a cita por esse nome. O app não lê arquivos da pessoa, não lê teclado nem mouse e não oferece controle remoto.
