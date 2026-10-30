# Research: Mostrar o que cada pessoa está transmitindo

## R1 — De onde vem o texto em cada caminho

| Caminho | Fonte do texto | Resultado |
|---|---|---|
| App, janela (005) | `CaptureSource.app` = `FileDescription` do executável (src-tauri/src/capture/sources.rs), o mesmo nome mostrado no cartão do seletor | "Hollow Knight", "Visual Studio Code", "Google Chrome"; vazio → "Janela" |
| App, tela (005) | `CaptureSource.title` ("Tela 1", "Tela 2") e a contagem de monitores em `sources` | 1 monitor → "Tela inteira"; 2+ → "Tela 2" |
| Site (004) | `track.getSettings().displaySurface` do que o navegador **entregou** (não o pedido) | `monitor` → "Tela inteira", `window` → "Janela", `browser` → "Aba do navegador", desconhecido → sem texto |
| Câmera (006) | `sourceTitle` já calculado (rótulo do aparelho ou "câmera frontal/traseira") | "Câmera · HD Pro Webcam C920", "Câmera frontal" |

**Decision**: uma função pura `liveLabel(...)` em `src/lib/live-label.ts` decide o texto a partir desses dados, testável sem tela. **O título da janela (`CaptureSource.title` de janelas) nunca entra** (Q1: A).

**Alternatives considered**: usar `track.label` no site. Rejeitado: no Chrome é algo como `window:12345:0`/`screen:0:0`, sem nome útil.

## R2 — Como o texto chega a quem assiste

**Decision**: campo opcional `label` na mensagem `live` (cliente → servidor) e em `LiveInfo` (servidor → todos, no `peer-live` e no `welcome`). O servidor valida e normaliza como o apelido (NFC, sem controle/direção/invisíveis, espaços juntos) com no máximo **60** caracteres. Inválido → descarta só o `label` e aceita o `live`.

**Rationale**: o "ao vivo" já é o canal (quem entra depois recebe no `welcome`; o `roster` da 011 já guarda o `live` inteiro e o apaga com a sala). Nada novo para guardar.

**Compatibilidade**: cliente antigo não manda `label` → viewers mostram só o nome, como hoje. Cliente antigo recebendo `label` em `LiveInfo` ignora o campo.

**Alternatives considered**: mandar o texto pelo canal `ctl` P2P. Rejeitado: quem entra depois e a lista "Na sala" dependem do servidor.

## R3 — Troca no meio (site, "compartilhar esta aba")

**Decision**: no `configurationchange` do vídeo, se o `displaySurface` mudou, recalcula e reenvia o `live` (o servidor só repassa). No app e na câmera, trocar de fonte já é parar e começar de novo.

## R4 — Onde mostrar

- **StreamTile**: no chip do nome (`AO VIVO · Lu`) acrescenta ` · Hollow Knight` em texto mais fraco, com reticências. Na miniatura (thumb) também.
- **Participant**: a linha de baixo "transmitindo" vira "transmitindo · Hollow Knight" (e "transmitindo pra sala · Hollow Knight" pra você).
- **Celular**: chips "Ao vivo agora" ganham o texto numa segunda linha pequena, cortada.
- **Você ao vivo**: o cartão mostra o mesmo texto que os outros veem (FR-007), no lugar do `sourceTitle` atual.
- **Não** vai pra `RoomPreview` (prévia da entrada) nem pra `RoomStatus` (Suas salas): o servidor monta essas respostas só com `live !== null` / contagens (FR-010).
